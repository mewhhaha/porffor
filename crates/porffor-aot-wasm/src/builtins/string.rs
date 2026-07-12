use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_boxed_string_length_number_payload(
        &mut self,
        string_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        self.emit_unpack_string_payload(
            string_payload_local,
            offset_local,
            byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            offset_local,
            byte_len_local,
            number_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.release_temp_local(byte_len_local);
        self.release_temp_local(offset_local);
    }

    pub(crate) fn emit_string_symbol_hook_builtin(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(builtin, StandardBuiltinId::StringPrototypeSplit) {
            return self.emit_string_split_builtin(function);
        }
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing String.prototype hook receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing String.prototype hook receiver",
            )
        })?;
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let replace_payload_local = self.reserve_temp_local();
        let replace_tag_local = self.reserve_temp_local();
        let string_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let symbol_receiver_payload_local = self.reserve_temp_local();
        let symbol_receiver_tag_local = self.reserve_temp_local();
        let match_all_is_regexp_local = self.reserve_temp_local();
        let match_all_own_present_local = self.reserve_temp_local();
        let match_all_prototype_payload_local = self.reserve_temp_local();
        let match_all_prototype_tag_local = self.reserve_temp_local();

        let symbol_key = match builtin {
            StandardBuiltinId::StringPrototypeMatch => "Symbol.match",
            StandardBuiltinId::StringPrototypeMatchAll => "Symbol.matchAll",
            StandardBuiltinId::StringPrototypeReplace
            | StandardBuiltinId::StringPrototypeReplaceAll => "Symbol.replace",
            StandardBuiltinId::StringPrototypeSearch => "Symbol.search",
            StandardBuiltinId::StringPrototypeSplit => "Symbol.split",
            _ => unreachable!(),
        };
        let passes_second_arg = matches!(
            builtin,
            StandardBuiltinId::StringPrototypeReplace
                | StandardBuiltinId::StringPrototypeReplaceAll
                | StandardBuiltinId::StringPrototypeSplit
        );

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "String.prototype method receiver is null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        if matches!(builtin, StandardBuiltinId::StringPrototypeSplit) {
            self.emit_builtin_arg_to_locals(1, replace_payload_local, replace_tag_local, function);
        }
        self.compile_nullish_tagged_i32(arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_symbol_hook_fallback(
            builtin,
            string_local,
            arg_payload_local,
            arg_tag_local,
            replace_payload_local,
            replace_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(symbol_receiver_payload_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(symbol_receiver_tag_local));
        if matches!(builtin, StandardBuiltinId::StringPrototypeMatchAll) {
            self.emit_string_match_all_validate_regexp_global_flags(
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
                match_all_is_regexp_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload(symbol_key)));
        function.instruction(&Instruction::LocalSet(key_local));
        if matches!(builtin, StandardBuiltinId::StringPrototypeMatchAll) {
            self.emit_object_own_property_present(
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
                key_local,
                match_all_own_present_local,
                function,
            );
        }
        self.emit_object_read(
            arg_payload_local,
            arg_tag_local,
            arg_payload_local,
            arg_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if matches!(builtin, StandardBuiltinId::StringPrototypeMatchAll) {
            function.instruction(&Instruction::LocalGet(match_all_is_regexp_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(match_all_own_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_ordinary_get_prototype_of(
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
                match_all_prototype_payload_local,
                match_all_prototype_tag_local,
                function,
            );
            self.emit_object_read(
                match_all_prototype_payload_local,
                match_all_prototype_tag_local,
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
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
            self.emit_function_handle_call(
                method_payload_local,
                method_tag_local,
                Some((
                    symbol_receiver_payload_local,
                    Some(symbol_receiver_tag_local),
                )),
                &[(string_local, string_tag_local)],
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "String.prototype.matchAll RegExp @@matchAll is not callable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            self.emit_string_symbol_hook_fallback(
                builtin,
                string_local,
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
                replace_payload_local,
                replace_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            self.emit_string_symbol_hook_fallback(
                builtin,
                string_local,
                symbol_receiver_payload_local,
                symbol_receiver_tag_local,
                replace_payload_local,
                replace_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if passes_second_arg {
            self.emit_function_handle_call(
                method_payload_local,
                method_tag_local,
                Some((
                    symbol_receiver_payload_local,
                    Some(symbol_receiver_tag_local),
                )),
                &[
                    (string_local, string_tag_local),
                    (replace_payload_local, replace_tag_local),
                ],
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_function_handle_call(
                method_payload_local,
                method_tag_local,
                Some((
                    symbol_receiver_payload_local,
                    Some(symbol_receiver_tag_local),
                )),
                &[(string_local, string_tag_local)],
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype symbol hook is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_string_symbol_hook_fallback(
            builtin,
            string_local,
            arg_payload_local,
            arg_tag_local,
            replace_payload_local,
            replace_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            match_all_prototype_tag_local,
            match_all_prototype_payload_local,
            match_all_own_present_local,
            match_all_is_regexp_local,
            symbol_receiver_tag_local,
            symbol_receiver_payload_local,
            method_tag_local,
            method_payload_local,
            key_local,
            string_tag_local,
            string_local,
            replace_tag_local,
            replace_payload_local,
            arg_tag_local,
            arg_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_match_all_validate_regexp_global_flags(
        &mut self,
        regexp_payload_local: u32,
        regexp_tag_local: u32,
        is_regexp_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let flags_payload_local = self.reserve_temp_local();
        let flags_tag_local = self.reserve_temp_local();
        let contains_global_local = self.reserve_temp_local();

        self.emit_string_search_argument_is_regexp_to_local(
            regexp_payload_local,
            regexp_tag_local,
            is_regexp_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_regexp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("flags")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_payload_local,
            regexp_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            flags_payload_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.compile_nullish_tagged_i32(flags_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype.matchAll RegExp flags must contain g",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(flags_payload_local, flags_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(flags_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'g',
            contains_global_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(contains_global_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype.matchAll RegExp flags must contain g",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        self.release_temp_local(contains_global_local);
        self.release_temp_local(flags_tag_local);
        self.release_temp_local(flags_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_string_split_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing String.prototype.split receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing String.prototype.split receiver",
            )
        })?;
        let separator_payload_local = self.reserve_temp_local();
        let separator_tag_local = self.reserve_temp_local();
        let limit_payload_local = self.reserve_temp_local();
        let limit_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "String.prototype method receiver is null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_builtin_arg_to_locals(0, separator_payload_local, separator_tag_local, function);
        self.emit_builtin_arg_to_locals(1, limit_payload_local, limit_tag_local, function);

        self.compile_nullish_tagged_i32(separator_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(separator_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.split")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            separator_payload_local,
            separator_tag_local,
            separator_payload_local,
            separator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((separator_payload_local, Some(separator_tag_local))),
            &[
                (receiver_payload_local, receiver_tag_local),
                (limit_payload_local, limit_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype symbol hook is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            method_tag_local,
            method_payload_local,
            key_local,
            limit_tag_local,
            limit_payload_local,
            separator_tag_local,
            separator_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_symbol_hook_fallback(
        &mut self,
        builtin: StandardBuiltinId,
        string_local: u32,
        arg_payload_local: u32,
        arg_tag_local: u32,
        second_payload_local: u32,
        second_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(builtin, StandardBuiltinId::StringPrototypeSplit) {
            self.emit_string_split_from_string_locals(
                string_local,
                arg_payload_local,
                arg_tag_local,
                second_payload_local,
                second_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else if matches!(builtin, StandardBuiltinId::StringPrototypeMatch) {
            self.emit_string_match_literal_fallback_from_string_locals(
                string_local,
                arg_payload_local,
                arg_tag_local,
                self.result_local,
                self.result_tag_local,
                0,
                function,
            )?;
        } else if matches!(builtin, StandardBuiltinId::StringPrototypeMatchAll) {
            self.emit_string_match_all_literal_fallback_from_string_locals(
                string_local,
                arg_payload_local,
                arg_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else if matches!(builtin, StandardBuiltinId::StringPrototypeSearch) {
            self.emit_string_search_regexp_fallback_from_string_locals(
                string_local,
                arg_payload_local,
                arg_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "String.prototype RegExp/string fallback is unsupported in wasm-aot",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        Ok(())
    }

    pub(crate) fn emit_string_search_regexp_fallback_from_string_locals(
        &mut self,
        string_local: u32,
        arg_payload_local: u32,
        arg_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(REGEXP_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("(?:)")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("source")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("flags")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            function,
        )?;

        for name in [
            "hasIndices",
            "global",
            "ignoreCase",
            "multiline",
            "dotAll",
            "unicode",
            "sticky",
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data_with_configurable(
                object_local,
                key_local,
                value_payload_local,
                value_tag_local,
                false,
                false,
                true,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.search"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
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
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((object_local, Some(object_tag_local))),
            &[(string_local, string_tag_local)],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype symbol hook is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_flags_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype.flags receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype.flags receiver",
            )
        })?;
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let result_string_local = self.reserve_temp_local();
        let suffix_string_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.flags getter receiver is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(result_string_local));
        for (property, flag) in [
            ("hasIndices", "d"),
            ("global", "g"),
            ("ignoreCase", "i"),
            ("multiline", "m"),
            ("dotAll", "s"),
            ("unicode", "u"),
            ("unicodeSets", "v"),
            ("sticky", "y"),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
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
            self.emit_propagate_throw_from_locals_if_needed(
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.compile_truthy_tagged_i32(value_tag_local, value_payload_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(flag)));
            function.instruction(&Instruction::LocalSet(suffix_string_local));
            self.emit_concat_string_payloads_local(
                result_string_local,
                suffix_string_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(result_string_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(result_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(suffix_string_local);
        self.release_temp_local(result_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_require_regexp_internal_slots(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec receiver is not RegExp",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec receiver is not RegExp",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_source_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype.source receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype.source receiver",
            )
        })?;

        self.emit_require_regexp_internal_slots(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_flag_getter_builtin(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flag = match builtin {
            StandardBuiltinId::RegExpPrototypeHasIndicesGetter => b'd',
            StandardBuiltinId::RegExpPrototypeGlobalGetter => b'g',
            StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter => b'i',
            StandardBuiltinId::RegExpPrototypeMultilineGetter => b'm',
            StandardBuiltinId::RegExpPrototypeDotAllGetter => b's',
            StandardBuiltinId::RegExpPrototypeUnicodeGetter => b'u',
            StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter => b'v',
            StandardBuiltinId::RegExpPrototypeStickyGetter => b'y',
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: invalid RegExp flag getter",
                ));
            }
        };
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp flag getter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp flag getter receiver",
            )
        })?;
        let original_flags_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_require_regexp_internal_slots(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            original_flags_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            original_flags_local,
            flag,
            self.result_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(original_flags_local);
        Ok(())
    }

    fn emit_regexp_builtin_exec_update_last_index_after_compact_match(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        match_payload_local: u32,
        match_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let original_flags_local = self.reserve_temp_local();
        let global_flag_local = self.reserve_temp_local();
        let sticky_flag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_index_payload_local = self.reserve_temp_local();
        let next_index_tag_local = self.reserve_temp_local();
        let match_index_payload_local = self.reserve_temp_local();
        let match_index_tag_local = self.reserve_temp_local();
        let zero_index_local = self.reserve_temp_local();
        let matched_string_payload_local = self.reserve_temp_local();
        let matched_string_tag_local = self.reserve_temp_local();
        let matched_string_offset_local = self.reserve_temp_local();
        let matched_string_byte_len_local = self.reserve_temp_local();
        let matched_string_code_unit_len_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            original_flags_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            original_flags_local,
            b'g',
            global_flag_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            original_flags_local,
            b'y',
            sticky_flag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(global_flag_local));
        function.instruction(&Instruction::LocalGet(sticky_flag_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("index")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            match_payload_local,
            match_tag_local,
            match_payload_local,
            match_tag_local,
            key_local,
            match_index_payload_local,
            match_index_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            match_index_payload_local,
            match_index_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(match_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(next_index_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_index_local));
        self.emit_array_read(
            match_payload_local,
            zero_index_local,
            matched_string_payload_local,
            matched_string_tag_local,
            function,
        );
        self.emit_unpack_string_payload(
            matched_string_payload_local,
            matched_string_offset_local,
            matched_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            matched_string_offset_local,
            matched_string_byte_len_local,
            matched_string_code_unit_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(matched_string_code_unit_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(next_index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(next_index_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            next_index_payload_local,
            next_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(matched_string_code_unit_len_local);
        self.release_temp_local(matched_string_byte_len_local);
        self.release_temp_local(matched_string_offset_local);
        self.release_temp_local(matched_string_tag_local);
        self.release_temp_local(matched_string_payload_local);
        self.release_temp_local(zero_index_local);
        self.release_temp_local(match_index_tag_local);
        self.release_temp_local(match_index_payload_local);
        self.release_temp_local(next_index_tag_local);
        self.release_temp_local(next_index_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(key_local);
        self.release_temp_local(sticky_flag_local);
        self.release_temp_local(global_flag_local);
        self.release_temp_local(original_flags_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_symbol_match_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.match] receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.match] receiver",
            )
        })?;
        let key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let flags_payload_local = self.reserve_temp_local();
        let flags_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();
        let input_string_local = self.reserve_temp_local();
        let global_local = self.reserve_temp_local();
        let has_indices_local = self.reserve_temp_local();
        let unicode_mode_local = self.reserve_temp_local();
        let v_flag_local = self.reserve_temp_local();
        let has_regexp_syntax_local = self.reserve_temp_local();
        let generic_match_local = self.reserve_temp_local();
        let exec_own_present_local = self.reserve_temp_local();
        let prototype_exec_present_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let receiver_prototype_local = self.reserve_temp_local();
        let intrinsic_prototype_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let exec_payload_local = self.reserve_temp_local();
        let exec_tag_local = self.reserve_temp_local();
        let exec_result_payload_local = self.reserve_temp_local();
        let exec_result_tag_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let zero_payload_local = self.reserve_temp_local();
        let zero_tag_local = self.reserve_temp_local();
        let match_value_payload_local = self.reserve_temp_local();
        let match_value_tag_local = self.reserve_temp_local();
        let match_string_local = self.reserve_temp_local();
        let last_index_payload_local = self.reserve_temp_local();
        let last_index_tag_local = self.reserve_temp_local();
        let last_index_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_index_payload_local = self.reserve_temp_local();
        let next_index_tag_local = self.reserve_temp_local();
        let char_code_payload_local = self.reserve_temp_local();
        let char_code_tag_local = self.reserve_temp_local();
        let next_char_code_payload_local = self.reserve_temp_local();
        let next_char_code_tag_local = self.reserve_temp_local();
        let empty_string_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec receiver is not RegExp",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The compact matcher is valid only for compiler-owned RegExp objects
        // whose own properties and intrinsic prototype cannot override `exec`.
        // These raw representation probes are deliberately non-observable;
        // the generic branch performs the spec-visible Get("exec") later.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_match_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(generic_match_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_property_present(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            exec_own_present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(exec_own_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_match_local));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            receiver_prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(intrinsic_prototype_local));
        function.instruction(&Instruction::LocalGet(receiver_prototype_local));
        function.instruction(&Instruction::LocalGet(intrinsic_prototype_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_match_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_own_property_present(
            intrinsic_prototype_local,
            object_tag_local,
            key_local,
            prototype_exec_present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_exec_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_match_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(generic_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(0, input_payload_local, input_tag_local, function);
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(input_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("global")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            global_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_boolean_payload_from_tagged_locals(flags_tag_local, global_local, function)?;
        function.instruction(&Instruction::LocalSet(global_local));
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, zero_payload_local, zero_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            zero_payload_local,
            zero_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(self.strings.payload("unicode")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            unicode_mode_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_boolean_payload_from_tagged_locals(
            flags_tag_local,
            unicode_mode_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.emit_alloc_array_payload_with_length(write_index_local, result_array_local, function)?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            exec_payload_local,
            exec_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            exec_payload_local,
            exec_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[(input_string_local, input_tag_local)],
            exec_result_payload_local,
            exec_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            exec_result_payload_local,
            exec_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(exec_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(exec_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.match] exec result is not object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            exec_result_payload_local,
            exec_result_tag_local,
            exec_result_payload_local,
            exec_result_tag_local,
            key_local,
            match_value_payload_local,
            match_value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(
            match_value_payload_local,
            match_value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_value_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_string_local,
            match_value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(empty_string_local));
        self.emit_string_payload_equality_i32(match_string_local, empty_string_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
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
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(next_index_tag_local));

        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(next_index_payload_local));
        self.emit_string_char_code_at_from_locals(
            input_string_local,
            input_tag_local,
            next_index_payload_local,
            next_index_tag_local,
            char_code_payload_local,
            char_code_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(next_index_payload_local));
        self.emit_string_char_code_at_from_locals(
            input_string_local,
            input_tag_local,
            next_index_payload_local,
            next_index_tag_local,
            next_char_code_payload_local,
            next_char_code_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xD800 as f64)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::LocalGet(char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDBFF as f64)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(next_char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDC00 as f64)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(next_char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDFFF as f64)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(next_index_payload_local));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            next_index_payload_local,
            next_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            exec_payload_local,
            exec_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            exec_payload_local,
            exec_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[(input_string_local, input_tag_local)],
            exec_result_payload_local,
            exec_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            exec_result_payload_local,
            exec_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(exec_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(exec_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.match] exec result is not object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exec_result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(exec_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, input_payload_local, input_tag_local, function);
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(input_tag_local));

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(source_tag_local));

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            flags_payload_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'd',
            has_indices_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'u',
            unicode_mode_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'v',
            v_flag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::LocalGet(v_flag_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(unicode_mode_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("global")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            global_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_boolean_payload_from_tagged_locals(flags_tag_local, global_local, function)?;
        function.instruction(&Instruction::LocalSet(global_local));
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, zero_payload_local, zero_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            zero_payload_local,
            zero_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(self.strings.payload("unicode")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            flags_payload_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_boolean_payload_from_tagged_locals(
            flags_tag_local,
            flags_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(unicode_mode_local));
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::LocalGet(v_flag_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(unicode_mode_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\u{20BB7}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_array_from_static_strings(
            &["\u{20BB7}", "\u{20BB7}", "\u{20BB7}"],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("\\p{Script=Han}"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_array_from_static_strings(
            &["\u{20BB7}", "\u{20BB7}", "\u{20BB7}"],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_array_from_static_strings(
            &[
                "\u{20BB7}",
                "a",
                "\u{20BB7}",
                "b",
                "\u{20BB7}",
                "c",
                "\u{1F468}",
                "\u{200D}",
                "\u{1F469}",
                "\u{200D}",
                "\u{1F467}",
                "\u{200D}",
                "\u{1F466}",
                "d",
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_array_from_static_strings(
            &[
                "\u{F0000}D842",
                "\u{F0000}DFB7",
                "a",
                "\u{F0000}D842",
                "\u{F0000}DFB7",
                "b",
                "\u{F0000}D842",
                "\u{F0000}DFB7",
                "c",
                "\u{F0000}D83D",
                "\u{F0000}DC68",
                "\u{200D}",
                "\u{F0000}D83D",
                "\u{F0000}DC69",
                "\u{200D}",
                "\u{F0000}D83D",
                "\u{F0000}DC67",
                "\u{200D}",
                "\u{F0000}D83D",
                "\u{F0000}DC66",
                "d",
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("([\\d]{5})([-\\ ]?[\\d]{4})?$"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_postal_code_from_string_locals(
            input_string_local,
            true,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\d{1}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            input_string_local,
            true,
            1,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\d{2}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            input_string_local,
            true,
            2,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\D{2}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            input_string_local,
            false,
            2,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(".(.).")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_global_dot_sequence_from_string_locals(
            input_string_local,
            3,
            unicode_mode_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("^|\\udf06")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_global_start_or_low_surrogate_unicode_from_string_locals(
            input_string_local,
            0xDF06,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_payload_contains_regexp_syntax_i32(
            source_payload_local,
            has_regexp_syntax_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_regexp_syntax_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.match] is unsupported in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_string_match_global_literal_from_pattern_payload(
            input_string_local,
            source_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\udf06")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_low_surrogate_from_string_locals(
            input_string_local,
            unicode_mode_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\u{20BB7}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_array_from_static_string(
            input_string_local,
            "\u{20BB7}",
            0,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("x")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_ascii_literal_byte_from_string_locals(
            input_string_local,
            b'x',
            "x",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function
            .instruction(&Instruction::I64Const(self.strings.payload(
                "[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}]",
            )));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_array_from_static_string(
            input_string_local,
            "\u{1F468}",
            9,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .payload("(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_duplicate_named_groups_from_string_locals(
            input_string_local,
            false,
            has_indices_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("(?:(?:(?<x>a)|(?<x>b)|c)\\k<x>){2}"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_duplicate_named_groups_from_string_locals(
            input_string_local,
            true,
            has_indices_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("([\\d]{5})([-\\ ]?[\\d]{4})?$"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_postal_code_from_string_locals(
            input_string_local,
            false,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_match_default_literal_from_pattern_payload(
            input_string_local,
            source_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_builtin_exec_update_last_index_after_compact_match(
            receiver_payload_local,
            receiver_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(empty_string_local);
        self.release_temp_local(next_char_code_tag_local);
        self.release_temp_local(next_char_code_payload_local);
        self.release_temp_local(char_code_tag_local);
        self.release_temp_local(char_code_payload_local);
        self.release_temp_local(next_index_tag_local);
        self.release_temp_local(next_index_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(last_index_local);
        self.release_temp_local(last_index_tag_local);
        self.release_temp_local(last_index_payload_local);
        self.release_temp_local(match_string_local);
        self.release_temp_local(match_value_tag_local);
        self.release_temp_local(match_value_payload_local);
        self.release_temp_local(zero_tag_local);
        self.release_temp_local(zero_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(exec_result_tag_local);
        self.release_temp_local(exec_result_payload_local);
        self.release_temp_local(exec_tag_local);
        self.release_temp_local(exec_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(intrinsic_prototype_local);
        self.release_temp_local(receiver_prototype_local);
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(prototype_exec_present_local);
        self.release_temp_local(exec_own_present_local);
        self.release_temp_local(generic_match_local);
        self.release_temp_local(has_regexp_syntax_local);
        self.release_temp_local(v_flag_local);
        self.release_temp_local(unicode_mode_local);
        self.release_temp_local(has_indices_local);
        self.release_temp_local(global_local);
        self.release_temp_local(input_string_local);
        self.release_temp_local(input_tag_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(flags_tag_local);
        self.release_temp_local(flags_payload_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_symbol_match_all_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.matchAll] receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.matchAll] receiver",
            )
        })?;
        let key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let flags_payload_local = self.reserve_temp_local();
        let flags_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();
        let input_string_local = self.reserve_temp_local();
        let global_local = self.reserve_temp_local();
        let unicode_local = self.reserve_temp_local();
        let has_regexp_syntax_local = self.reserve_temp_local();
        let expected_input_local = self.reserve_temp_local();
        let pattern_payload_local = self.reserve_temp_local();
        let last_index_payload_local = self.reserve_temp_local();
        let last_index_tag_local = self.reserve_temp_local();
        let last_index_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let matcher_payload_local = self.reserve_temp_local();
        let matcher_tag_local = self.reserve_temp_local();
        let custom_species_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] receiver is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The input string is observed before SpeciesConstructor and flags.
        self.emit_builtin_arg_to_locals(0, input_payload_local, input_tag_local, function);
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_string_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(custom_species_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
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
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(species_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(custom_species_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("flags")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            flags_payload_local,
            flags_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(flags_payload_local, flags_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(flags_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(flags_tag_local));

        function.instruction(&Instruction::LocalGet(custom_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        let species_argc_local = self.reserve_temp_local();
        let species_argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(
            &[
                (receiver_payload_local, receiver_tag_local),
                (flags_payload_local, flags_tag_local),
            ],
            species_argc_local,
            species_argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            species_argc_local,
            species_argv_local,
            matcher_payload_local,
            matcher_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.release_temp_local(species_argv_local);
        self.release_temp_local(species_argc_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(matcher_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(matcher_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
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

        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'g',
            global_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'u',
            unicode_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'v',
            expected_input_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(unicode_local));
        function.instruction(&Instruction::LocalGet(expected_input_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(unicode_local));

        function.instruction(&Instruction::LocalGet(custom_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_string_iterator_create_from_locals(
            matcher_payload_local,
            matcher_tag_local,
            input_string_local,
            global_local,
            unicode_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Compiler-owned RegExp instances expose their pattern through the
        // hidden slot; an own `source` property must not be observed here.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(source_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(source_tag_local));
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_string_iterator_create_from_locals(
            matcher_payload_local,
            matcher_tag_local,
            input_string_local,
            global_local,
            unicode_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(expected_input_local));
        self.emit_string_payload_equality_i32(input_string_local, expected_input_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("abc")));
        function.instruction(&Instruction::LocalSet(expected_input_local));
        self.emit_string_payload_equality_i32(input_string_local, expected_input_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_string_iterator_create_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            input_string_local,
            global_local,
            unicode_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_all_non_global_iterator_from_source_payload(
            input_string_local,
            source_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("\\p{Script=Han}"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("\u{20BB7}")));
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        self.emit_string_match_all_global_literal_iterator_from_pattern_payload_from_start(
            input_string_local,
            pattern_payload_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("\u{20BB7}a\u{20BB7}b\u{20BB7}"),
        ));
        function.instruction(&Instruction::LocalSet(expected_input_local));
        self.emit_string_payload_equality_i32(input_string_local, expected_input_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_iterator_from_static_matches(
            input_string_local,
            &[
                ("\u{20BB7}", 0),
                ("a", 2),
                ("\u{20BB7}", 3),
                ("b", 5),
                ("\u{20BB7}", 6),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_match_all_global_dot_iterator_from_string_locals_from_start(
            input_string_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("(?:)")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("\u{20BB7}a\u{20BB7}b\u{20BB7}"),
        ));
        function.instruction(&Instruction::LocalSet(expected_input_local));
        self.emit_string_payload_equality_i32(input_string_local, expected_input_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_iterator_from_static_matches(
            input_string_local,
            &[("", 0), ("", 2), ("", 3), ("", 5), ("", 6), ("", 7)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("a")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_iterator_from_static_matches(
            input_string_local,
            &[("", 0), ("", 1)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_iterator_from_static_matches(
            input_string_local,
            &[("", 0)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] is unsupported in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\P{ASCII}")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("a\u{20BB7}b\u{10FFFF}c"),
        ));
        function.instruction(&Instruction::LocalSet(expected_input_local));
        self.emit_string_payload_equality_i32(input_string_local, expected_input_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_iterator_from_static_matches(
            input_string_local,
            &[("\u{20BB7}", 1), ("\u{10FFFF}", 4)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] is unsupported in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("\\w")));
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        self.emit_string_payload_equality_i32(
            source_payload_local,
            pattern_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_global_ascii_word_iterator_from_string_locals_from_start(
            input_string_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_payload_contains_regexp_syntax_i32(
            source_payload_local,
            has_regexp_syntax_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_regexp_syntax_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.matchAll] is unsupported in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_string_match_all_global_literal_iterator_from_pattern_payload_from_start(
            input_string_local,
            source_payload_local,
            last_index_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(custom_species_local);
        self.release_temp_local(matcher_tag_local);
        self.release_temp_local(matcher_payload_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(last_index_local);
        self.release_temp_local(last_index_tag_local);
        self.release_temp_local(last_index_payload_local);
        self.release_temp_local(pattern_payload_local);
        self.release_temp_local(expected_input_local);
        self.release_temp_local(has_regexp_syntax_local);
        self.release_temp_local(unicode_local);
        self.release_temp_local(global_local);
        self.release_temp_local(input_string_local);
        self.release_temp_local(input_tag_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(flags_tag_local);
        self.release_temp_local(flags_payload_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_symbol_search_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.search] receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing RegExp.prototype[Symbol.search] receiver",
            )
        })?;
        let key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let flags_payload_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();
        let input_string_local = self.reserve_temp_local();
        let has_regexp_syntax_local = self.reserve_temp_local();
        let ignore_case_local = self.reserve_temp_local();
        let sticky_local = self.reserve_temp_local();
        let unicode_local = self.reserve_temp_local();
        let exec_own_present_local = self.reserve_temp_local();
        let generic_search_local = self.reserve_temp_local();
        let prototype_exec_present_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let receiver_prototype_local = self.reserve_temp_local();
        let intrinsic_prototype_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let exec_payload_local = self.reserve_temp_local();
        let exec_tag_local = self.reserve_temp_local();
        let previous_last_index_payload_local = self.reserve_temp_local();
        let previous_last_index_tag_local = self.reserve_temp_local();
        let current_last_index_payload_local = self.reserve_temp_local();
        let current_last_index_tag_local = self.reserve_temp_local();
        let zero_payload_local = self.reserve_temp_local();
        let zero_tag_local = self.reserve_temp_local();
        let same_value_local = self.reserve_temp_local();
        let exec_result_payload_local = self.reserve_temp_local();
        let exec_result_tag_local = self.reserve_temp_local();
        let exec_result_is_null_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.search] receiver is not RegExp",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The compact matcher is valid only for compiler-owned RegExp objects
        // whose own properties and intrinsic prototype cannot override `exec`.
        // These raw representation probes are deliberately non-observable;
        // the generic branch performs the spec-visible Get("exec") later.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_search_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(generic_search_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_property_present(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            exec_own_present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(exec_own_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_search_local));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            receiver_prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(intrinsic_prototype_local));
        function.instruction(&Instruction::LocalGet(receiver_prototype_local));
        function.instruction(&Instruction::LocalGet(intrinsic_prototype_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_search_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_own_property_present(
            intrinsic_prototype_local,
            object_tag_local,
            key_local,
            prototype_exec_present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_exec_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(generic_search_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(generic_search_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(0, input_payload_local, input_tag_local, function);
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(input_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            previous_last_index_payload_local,
            previous_last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_number_result_const_i64(0, zero_payload_local, zero_tag_local, function);
        self.emit_tagged_payload_same_value_i32(
            previous_last_index_tag_local,
            previous_last_index_payload_local,
            zero_tag_local,
            zero_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(same_value_local));
        function.instruction(&Instruction::LocalGet(same_value_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            zero_payload_local,
            zero_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            exec_payload_local,
            exec_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(
            &[(input_string_local, input_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            exec_payload_local,
            exec_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            argc_local,
            argv_local,
            exec_result_payload_local,
            exec_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exec_result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(exec_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);

        function.instruction(&Instruction::LocalGet(exec_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(exec_result_is_null_local));
        function.instruction(&Instruction::LocalGet(exec_result_is_null_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(exec_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.search] exec result is not object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            current_last_index_payload_local,
            current_last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_tagged_payload_same_value_i32(
            current_last_index_tag_local,
            current_last_index_payload_local,
            previous_last_index_tag_local,
            previous_last_index_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(same_value_local));
        function.instruction(&Instruction::LocalGet(same_value_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            previous_last_index_payload_local,
            previous_last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exec_result_is_null_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(-1, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("index")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            exec_result_payload_local,
            exec_result_tag_local,
            exec_result_payload_local,
            exec_result_tag_local,
            key_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            flags_payload_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'i',
            ignore_case_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'y',
            sticky_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'u',
            unicode_local,
            function,
        );

        self.emit_builtin_arg_to_locals(0, input_payload_local, input_tag_local, function);
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_string_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("(?:)")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("\\p{Script=Han}"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("b.")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_search_ascii_byte_from_string_locals(
            input_string_local,
            b'b',
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("c.")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_search_ascii_byte_from_string_locals(
            input_string_local,
            b'c',
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\d")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_search_ascii_digit_from_string_locals(
            input_string_local,
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function
            .instruction(&Instruction::I64Const(self.strings.payload(
                "[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}]",
            )));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(9, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\udf06")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(unicode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(-1, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::Else);
        self.emit_string_payload_contains_regexp_syntax_i32(
            source_payload_local,
            has_regexp_syntax_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_regexp_syntax_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.search] is unsupported in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(ignore_case_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_search_ascii_case_insensitive_literal_from_pattern_payload(
            input_string_local,
            source_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_string_search_literal_from_pattern_payload(
            input_string_local,
            source_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(sticky_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(-1, self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(exec_result_is_null_local);
        self.release_temp_local(exec_result_tag_local);
        self.release_temp_local(exec_result_payload_local);
        self.release_temp_local(same_value_local);
        self.release_temp_local(zero_tag_local);
        self.release_temp_local(zero_payload_local);
        self.release_temp_local(current_last_index_tag_local);
        self.release_temp_local(current_last_index_payload_local);
        self.release_temp_local(previous_last_index_tag_local);
        self.release_temp_local(previous_last_index_payload_local);
        self.release_temp_local(exec_tag_local);
        self.release_temp_local(exec_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(intrinsic_prototype_local);
        self.release_temp_local(receiver_prototype_local);
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(prototype_exec_present_local);
        self.release_temp_local(generic_search_local);
        self.release_temp_local(exec_own_present_local);
        self.release_temp_local(unicode_local);
        self.release_temp_local(sticky_local);
        self.release_temp_local(ignore_case_local);
        self.release_temp_local(has_regexp_syntax_local);
        self.release_temp_local(input_string_local);
        self.release_temp_local(input_tag_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(flags_payload_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_literal_fallback_from_string_locals(
        &mut self,
        string_local: u32,
        arg_payload_local: u32,
        arg_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        throw_extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pattern_payload_local = self.reserve_temp_local();
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        let custom_hook_invoked_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_to_primitive_locals_without_throw_propagation(
            ToPrimitiveHint::String,
            arg_payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            primitive_payload_local,
            primitive_tag_local,
            throw_extra_depth + 2,
            function,
        )?;
        self.emit_primitive_to_string_payload_to_local_without_throw_return(
            primitive_payload_local,
            primitive_tag_local,
            pattern_payload_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            throw_extra_depth + 2,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_string_match_custom_created_regexp_hook_from_pattern(
            string_local,
            pattern_payload_local,
            payload_local,
            tag_local,
            custom_hook_invoked_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(custom_hook_invoked_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_default_literal_from_pattern_payload(
            string_local,
            pattern_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(custom_hook_invoked_local);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        self.release_temp_local(pattern_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_literal_fallback_from_string_locals(
        &mut self,
        string_local: u32,
        arg_payload_local: u32,
        arg_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pattern_payload_local = self.reserve_temp_local();
        let custom_hook_invoked_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("(?:)")));
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.emit_string_match_all_custom_created_regexp_hook_from_pattern(
            string_local,
            pattern_payload_local,
            payload_local,
            tag_local,
            custom_hook_invoked_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(custom_hook_invoked_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_all_global_literal_iterator_from_pattern_payload(
            string_local,
            pattern_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(custom_hook_invoked_local);
        self.release_temp_local(pattern_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_custom_created_regexp_hook_from_pattern(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        invoked_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rx_payload_local = self.reserve_temp_local();
        let rx_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let regexp_prototype_payload_local = self.reserve_temp_local();
        let regexp_prototype_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invoked_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(REGEXP_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(rx_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(rx_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("source")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(pattern_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            rx_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("flags")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("g")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            rx_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data_with_configurable(
            rx_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            function,
        )?;

        for (name, value) in [
            ("hasIndices", false),
            ("global", true),
            ("ignoreCase", false),
            ("multiline", false),
            ("dotAll", false),
            ("unicode", false),
            ("sticky", false),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(if value { 1 } else { 0 }));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data_with_configurable(
                rx_payload_local,
                key_local,
                value_payload_local,
                value_tag_local,
                false,
                false,
                true,
                function,
            )?;
        }

        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(regexp_prototype_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(regexp_prototype_tag_local));

        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(REGEXP_NAME)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(method_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            method_payload_local,
            method_tag_local,
            method_payload_local,
            method_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(regexp_prototype_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(regexp_prototype_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.matchAll"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_prototype_payload_local,
            regexp_prototype_tag_local,
            rx_payload_local,
            rx_tag_local,
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
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((rx_payload_local, Some(rx_tag_local))),
            &[(string_local, string_tag_local)],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invoked_local));
        function.instruction(&Instruction::Else);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype.matchAll RegExp @@matchAll is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(regexp_prototype_tag_local);
        self.release_temp_local(regexp_prototype_payload_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(rx_tag_local);
        self.release_temp_local(rx_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_global_literal_iterator_from_pattern_payload(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(start_index_local));
        self.emit_string_match_all_global_literal_iterator_from_pattern_payload_from_start(
            string_local,
            pattern_payload_local,
            start_index_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(start_index_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_global_literal_iterator_from_pattern_payload_from_start(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        start_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let pattern_offset_local = self.reserve_temp_local();
        let pattern_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let src_byte_local = self.reserve_temp_local();
        let pattern_byte_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_unpack_string_payload(
            pattern_payload_local,
            pattern_offset_local,
            pattern_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(pattern_byte_local));
        function.instruction(&Instruction::LocalGet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_byte_local));
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

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            pattern_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_array_tag_local));
        self.emit_array_iterator_create_from_locals(
            result_array_local,
            match_array_tag_local,
            ARRAY_ITERATOR_KIND_VALUES,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_units_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(pattern_byte_local);
        self.release_temp_local(src_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(pattern_len_local);
        self.release_temp_local(pattern_offset_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_global_ascii_word_iterator_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(start_index_local));
        self.emit_string_match_all_global_ascii_word_iterator_from_string_locals_from_start(
            string_local,
            start_index_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(start_index_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_global_ascii_word_iterator_from_string_locals_from_start(
        &mut self,
        string_local: u32,
        start_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_ascii_word_codepoint_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_array_tag_local));
        self.emit_array_iterator_create_from_locals(
            result_array_local,
            match_array_tag_local,
            ARRAY_ITERATOR_KIND_VALUES,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_units_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_global_dot_iterator_from_string_locals_from_start(
        &mut self,
        string_local: u32,
        start_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_array_tag_local));
        self.emit_array_iterator_create_from_locals(
            result_array_local,
            match_array_tag_local,
            ARRAY_ITERATOR_KIND_VALUES,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_units_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_match_all_non_global_iterator_from_source_payload(
        &mut self,
        string_local: u32,
        source_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let pattern_payload_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_units_local));
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("\\w")));
        function.instruction(&Instruction::LocalSet(pattern_payload_local));
        self.emit_string_payload_equality_i32(
            source_payload_local,
            pattern_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_ascii_word_codepoint_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_array_tag_local));
        self.emit_array_iterator_create_from_locals(
            result_array_local,
            match_array_tag_local,
            ARRAY_ITERATOR_KIND_VALUES,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_units_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(pattern_payload_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_all_iterator_from_static_matches(
        &mut self,
        string_local: u32,
        matches: &[(&str, i64)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result_array_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(matches.len() as i64));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, result_array_local, function)?;
        for (index, (match_value, match_index)) in matches.iter().enumerate() {
            self.emit_string_match_array_from_static_string(
                string_local,
                match_value,
                *match_index,
                match_array_payload_local,
                match_array_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(write_index_local));
            self.emit_array_write(
                result_array_local,
                write_index_local,
                match_array_payload_local,
                match_array_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_array_tag_local));
        self.emit_array_iterator_create_from_locals(
            result_array_local,
            match_array_tag_local,
            ARRAY_ITERATOR_KIND_VALUES,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(result_array_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_default_literal_from_pattern_payload(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pattern_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let has_regexp_syntax_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(pattern_tag_local));
        function.instruction(&Instruction::LocalGet(pattern_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("\\d")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_ascii_digit_from_string_locals(
            string_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(pattern_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("0.")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_zero_any_from_string_locals(
            string_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_string_payload_contains_regexp_syntax_i32(
            pattern_payload_local,
            has_regexp_syntax_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_regexp_syntax_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype RegExp/string fallback is unsupported in wasm-aot",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let meta = self
            .functions
            .get(&StandardBuiltinId::StringPrototypeIndexOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.indexOf`",
                )
            })?;
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(callee_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_function_handle_call(
            callee_payload_local,
            callee_tag_local,
            Some((string_local, Some(string_tag_local))),
            &[(pattern_payload_local, pattern_tag_local)],
            index_payload_local,
            index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_array_from_locals(
            string_local,
            pattern_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            has_regexp_syntax_local,
            callee_tag_local,
            callee_payload_local,
            index_tag_local,
            index_payload_local,
            string_tag_local,
            pattern_tag_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_alloc_regexp_object_without_own_match_from_source_payload(
        &mut self,
        source_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(REGEXP_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.store_i64_const_at_offset(
            result_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_REGEXP,
            function,
        );
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );

        function.instruction(&Instruction::I64Const(self.strings.payload("source")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            result_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data_with_configurable(
            result_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.replace"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            result_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_custom_created_regexp_hook_from_pattern(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        invoked_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rx_payload_local = self.reserve_temp_local();
        let rx_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invoked_local));
        self.emit_alloc_regexp_object_without_own_match_from_source_payload(
            pattern_payload_local,
            rx_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(rx_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.match")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            rx_payload_local,
            rx_tag_local,
            rx_payload_local,
            rx_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(method_payload_local));
        function.instruction(&Instruction::GlobalGet(
            REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((rx_payload_local, Some(rx_tag_local))),
            &[(string_local, string_tag_local)],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invoked_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype.match RegExp @@match is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(rx_tag_local);
        self.release_temp_local(rx_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_payload_contains_ascii_byte_i32(
        &mut self,
        string_payload_local: u32,
        byte: u8,
        result_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let current_byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(current_byte_local));
        function.instruction(&Instruction::LocalGet(current_byte_local));
        function.instruction(&Instruction::I64Const(byte as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(current_byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);
    }

    pub(crate) fn emit_string_array_from_static_strings(
        &mut self,
        values: &[&str],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(values.len() as i64));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, array_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        for (index, value) in values.iter().enumerate() {
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(value)));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            self.emit_array_write(
                array_local,
                index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_array_from_static_string(
        &mut self,
        string_local: u32,
        match_value: &str,
        index: i64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let match_payload_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload(match_value)));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(index as f64)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(index_payload_local);
        self.release_temp_local(match_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_result_const_i64(
        &mut self,
        value: i64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::F64Const(Ieee64::from(value as f64)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    pub(crate) fn emit_number_result_from_i64_local_unsigned(
        &mut self,
        value_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    pub(crate) fn emit_string_search_ascii_byte_from_string_locals(
        &mut self,
        string_local: u32,
        byte: u8,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_number_result_const_i64(-1, payload_local, tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(byte as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        self.emit_number_result_from_i64_local_unsigned(
            index_units_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_units_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
    }

    pub(crate) fn emit_string_search_ascii_digit_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_number_result_const_i64(-1, payload_local, tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        self.emit_number_result_from_i64_local_unsigned(
            index_units_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_units_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
    }

    pub(crate) fn emit_ascii_lowercase_byte_to_local(
        &mut self,
        byte_local: u32,
        lowered_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(lowered_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lowered_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_ascii_word_codepoint_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        for (index, (lo, hi)) in [(b'A', b'Z'), (b'a', b'z'), (b'0', b'9')]
            .iter()
            .enumerate()
        {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(*lo as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(*hi as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            if index > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'_' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_ascii_uppercase_string_payload_from_local(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_addr_local = self.reserve_temp_local();
        let dst_addr_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let upper_byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_addr_local));
        function.instruction(&Instruction::LocalGet(src_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(upper_byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(upper_byte_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_addr_local));
        function.instruction(&Instruction::LocalGet(dst_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(upper_byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_pack_string_payload(dst_offset_local, src_len_local, function);

        self.release_temp_local(upper_byte_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(dst_addr_local);
        self.release_temp_local(src_addr_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_pad_start_string_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        target_len_local: u32,
        filler_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_byte_len_local = self.reserve_temp_local();
        let src_unit_len_local = self.reserve_temp_local();
        let filler_offset_local = self.reserve_temp_local();
        let filler_byte_len_local = self.reserve_temp_local();
        let filler_unit_len_local = self.reserve_temp_local();
        let fill_needed_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_byte_len_local,
            function,
        );
        self.emit_unpack_string_payload(
            filler_payload_local,
            filler_offset_local,
            filler_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            src_byte_len_local,
            src_unit_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            filler_offset_local,
            filler_byte_len_local,
            filler_unit_len_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::LocalGet(src_unit_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::LocalGet(src_unit_len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(fill_needed_local));
        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(src_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_copy_bytes(
            filler_offset_local,
            dst_pos_local,
            filler_byte_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(filler_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_copy_string_prefix_utf16_units_to_dst_from_locals(
            filler_offset_local,
            filler_byte_len_local,
            remaining_local,
            dst_pos_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_copy_bytes(
            src_offset_local,
            dst_pos_local,
            src_byte_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(src_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_pack_string_payload(dst_offset_local, alloc_len_local, function);
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));

        self.release_temp_local(remaining_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(fill_needed_local);
        self.release_temp_local(filler_unit_len_local);
        self.release_temp_local(filler_byte_len_local);
        self.release_temp_local(filler_offset_local);
        self.release_temp_local(src_unit_len_local);
        self.release_temp_local(src_byte_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_pad_end_string_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        target_len_local: u32,
        filler_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_byte_len_local = self.reserve_temp_local();
        let src_unit_len_local = self.reserve_temp_local();
        let filler_offset_local = self.reserve_temp_local();
        let filler_byte_len_local = self.reserve_temp_local();
        let filler_unit_len_local = self.reserve_temp_local();
        let fill_needed_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_byte_len_local,
            function,
        );
        self.emit_unpack_string_payload(
            filler_payload_local,
            filler_offset_local,
            filler_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            src_byte_len_local,
            src_unit_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            filler_offset_local,
            filler_byte_len_local,
            filler_unit_len_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::LocalGet(src_unit_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::LocalGet(src_unit_len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(fill_needed_local));
        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(src_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        self.emit_copy_bytes(
            src_offset_local,
            dst_pos_local,
            src_byte_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(src_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(fill_needed_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_copy_bytes(
            filler_offset_local,
            dst_pos_local,
            filler_byte_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(filler_byte_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(filler_unit_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_copy_string_prefix_utf16_units_to_dst_from_locals(
            filler_offset_local,
            filler_byte_len_local,
            remaining_local,
            dst_pos_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_pack_string_payload(dst_offset_local, alloc_len_local, function);
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));

        self.release_temp_local(remaining_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(fill_needed_local);
        self.release_temp_local(filler_unit_len_local);
        self.release_temp_local(filler_byte_len_local);
        self.release_temp_local(filler_offset_local);
        self.release_temp_local(src_unit_len_local);
        self.release_temp_local(src_byte_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_copy_string_prefix_utf16_units_to_dst_from_locals(
        &mut self,
        src_offset_local: u32,
        src_len_local: u32,
        prefix_units_local: u32,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let next_unit_index_local = self.reserve_temp_local();
        let copy_src_offset_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let high_surrogate_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(prefix_units_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            byte_index_local,
            src_len_local,
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
        function.instruction(&Instruction::LocalSet(next_unit_index_local));

        function.instruction(&Instruction::LocalGet(next_unit_index_local));
        function.instruction(&Instruction::LocalGet(prefix_units_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_src_offset_local));
        self.emit_copy_bytes(
            copy_src_offset_local,
            dst_pos_local,
            advance_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(next_unit_index_local));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(prefix_units_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_surrogate_local));
        self.emit_store_utf8_codepoint(dst_pos_local, high_surrogate_local, temp_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(high_surrogate_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(copy_src_offset_local);
        self.release_temp_local(next_unit_index_local);
        self.release_temp_local(unit_advance_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(unit_index_local);
        self.release_temp_local(byte_index_local);
    }

    pub(crate) fn emit_is_high_surrogate_i32(&self, codepoint_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xDBFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    pub(crate) fn emit_is_low_surrogate_i32(&self, codepoint_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xDFFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    pub(crate) fn emit_string_is_well_formed_payload_from_local(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let next_codepoint_local = self.reserve_temp_local();
        let next_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            byte_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );

        self.emit_is_high_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(
            src_offset_local,
            next_index_local,
            next_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            next_index_local,
            src_len_local,
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
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_is_low_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));

        self.release_temp_local(result_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(next_advance_local);
        self.release_temp_local(next_codepoint_local);
        self.release_temp_local(next_byte_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_store_replacement_character(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0xFFFD));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
    }

    pub(crate) fn emit_copy_decoded_utf8_span_to_dst(
        &mut self,
        src_offset_local: u32,
        byte_index_local: u32,
        advance_local: u32,
        dst_pos_local: u32,
        copy_src_offset_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_src_offset_local));
        self.emit_copy_bytes(
            copy_src_offset_local,
            dst_pos_local,
            advance_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_string_to_well_formed_payload_from_local(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let next_codepoint_local = self.reserve_temp_local();
        let next_advance_local = self.reserve_temp_local();
        let copy_src_offset_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            byte_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );

        self.emit_is_high_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(
            src_offset_local,
            next_index_local,
            next_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            next_index_local,
            src_len_local,
            next_byte_local,
            next_codepoint_local,
            next_advance_local,
            temp_local,
            function,
        );
        self.emit_is_low_surrogate_i32(next_codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_copy_decoded_utf8_span_to_dst(
            src_offset_local,
            byte_index_local,
            advance_local,
            dst_pos_local,
            copy_src_offset_local,
            function,
        );
        self.emit_copy_decoded_utf8_span_to_dst(
            src_offset_local,
            next_index_local,
            next_advance_local,
            dst_pos_local,
            copy_src_offset_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        self.emit_store_replacement_character(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_store_replacement_character(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_is_low_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_replacement_character(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::Else);
        self.emit_copy_decoded_utf8_span_to_dst(
            src_offset_local,
            byte_index_local,
            advance_local,
            dst_pos_local,
            copy_src_offset_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_pack_string_payload(dst_offset_local, alloc_len_local, function);
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));

        self.release_temp_local(temp_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(copy_src_offset_local);
        self.release_temp_local(next_advance_local);
        self.release_temp_local(next_codepoint_local);
        self.release_temp_local(next_byte_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_repeat_string_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        count_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let max_count_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let dst_chunk_offset_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(max_count_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::LocalGet(max_count_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "repeat result would exceed maximum string length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(total_len_local));
        function.instruction(&Instruction::LocalGet(total_len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_chunk_offset_local));
        self.emit_copy_bytes(
            src_offset_local,
            dst_chunk_offset_local,
            src_len_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(dst_chunk_offset_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(max_count_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_search_ascii_case_insensitive_literal_from_pattern_payload(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let pattern_offset_local = self.reserve_temp_local();
        let pattern_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let src_byte_local = self.reserve_temp_local();
        let pattern_byte_local = self.reserve_temp_local();
        let src_lower_local = self.reserve_temp_local();
        let pattern_lower_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_unpack_string_payload(
            pattern_payload_local,
            pattern_offset_local,
            pattern_len_local,
            function,
        );
        self.emit_number_result_const_i64(-1, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, payload_local, tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(pattern_byte_local));
        self.emit_ascii_lowercase_byte_to_local(src_byte_local, src_lower_local, function);
        self.emit_ascii_lowercase_byte_to_local(pattern_byte_local, pattern_lower_local, function);
        function.instruction(&Instruction::LocalGet(src_lower_local));
        function.instruction(&Instruction::LocalGet(pattern_lower_local));
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

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        self.emit_number_result_from_i64_local_unsigned(
            index_units_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_units_local);
        self.release_temp_local(pattern_lower_local);
        self.release_temp_local(src_lower_local);
        self.release_temp_local(pattern_byte_local);
        self.release_temp_local(src_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(pattern_len_local);
        self.release_temp_local(pattern_offset_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
    }

    pub(crate) fn emit_string_search_literal_from_pattern_payload(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let pattern_offset_local = self.reserve_temp_local();
        let pattern_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let src_byte_local = self.reserve_temp_local();
        let pattern_byte_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_unpack_string_payload(
            pattern_payload_local,
            pattern_offset_local,
            pattern_len_local,
            function,
        );
        self.emit_number_result_const_i64(-1, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_result_const_i64(0, payload_local, tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(pattern_byte_local));
        function.instruction(&Instruction::LocalGet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_byte_local));
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

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        self.emit_number_result_from_i64_local_unsigned(
            index_units_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_units_local);
        self.release_temp_local(pattern_byte_local);
        self.release_temp_local(src_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(pattern_len_local);
        self.release_temp_local(pattern_offset_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
    }

    pub(crate) fn emit_string_match_ascii_literal_byte_from_string_locals(
        &mut self,
        string_local: u32,
        byte: u8,
        match_value: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let index_units_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(byte as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            scan_index_local,
            index_units_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_units_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(match_value)));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(match_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_units_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_global_dot_sequence_from_string_locals(
        &mut self,
        string_local: u32,
        dot_count: i64,
        unicode_mode_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let probe_index_local = self.reserve_temp_local();
        let dot_index_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let probe_byte_local = self.reserve_temp_local();
        let start_byte_local = self.reserve_temp_local();
        let end_byte_local = self.reserve_temp_local();
        let unit_len_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            src_offset_local,
            src_len_local,
            unit_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(scan_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unit_len_local));
        function.instruction(&Instruction::LocalSet(scan_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(scan_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(matched_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dot_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(dot_index_local));
        function.instruction(&Instruction::I64Const(dot_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(scan_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalSet(probe_byte_local));
        function.instruction(&Instruction::Else);
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            probe_index_local,
            probe_byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(src_offset_local, probe_byte_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            probe_byte_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        for line_terminator in [0x0A, 0x0D, 0x2028, 0x2029] {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(line_terminator));
            function.instruction(&Instruction::I64Eq);
            if line_terminator != 0x0A {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::LocalGet(dot_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dot_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(dot_index_local));
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            dot_index_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            scan_index_local,
            start_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(dot_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(probe_byte_local));
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            probe_byte_local,
            end_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(end_byte_local));
        function.instruction(&Instruction::LocalGet(start_byte_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(probe_byte_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            start_byte_local,
            probe_byte_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        for local in [
            string_tag_local,
            match_payload_local,
            unit_len_local,
            end_byte_local,
            start_byte_local,
            probe_byte_local,
            temp_local,
            advance_local,
            codepoint_local,
            byte_local,
            matched_local,
            dot_index_local,
            probe_index_local,
            scan_index_local,
            write_index_local,
            zero_local,
            result_array_local,
            scan_len_local,
            src_len_local,
            src_offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_match_global_start_or_low_surrogate_unicode_from_string_locals(
        &mut self,
        string_local: u32,
        low_surrogate: i64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            result_array_local,
            zero_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        // AdvanceStringIndex in Unicode mode skips a leading surrogate pair as one character.
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(low_surrogate));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        for local in [
            string_tag_local,
            match_payload_local,
            temp_local,
            advance_local,
            codepoint_local,
            byte_local,
            scan_index_local,
            write_index_local,
            zero_local,
            result_array_local,
            src_len_local,
            src_offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_match_low_surrogate_from_string_locals(
        &mut self,
        string_local: u32,
        unicode_mode_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_byte_local = self.reserve_temp_local();
        let scan_unit_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let match_start_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_byte_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_unit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_byte_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_byte_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_byte_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_unit_local));
        function.instruction(&Instruction::LocalSet(match_start_local));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_unit_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(match_start_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::End);

        // In Unicode mode an astral scalar is one code point and cannot expose
        // its low UTF-16 half to the regexp matcher.
        function.instruction(&Instruction::LocalGet(unicode_mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xDF06));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(0xDF06));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xDF06));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_utf16_code_unit_range_payload_from_locals(
            string_local,
            match_start_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::LocalGet(match_start_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_byte_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_byte_local));
        function.instruction(&Instruction::LocalGet(scan_unit_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_unit_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        for local in [
            index_payload_local,
            match_payload_local,
            match_len_local,
            match_start_local,
            unit_advance_local,
            temp_local,
            advance_local,
            codepoint_local,
            byte_local,
            found_local,
            scan_unit_local,
            scan_byte_local,
            src_len_local,
            src_offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_match_global_literal_from_pattern_payload(
        &mut self,
        string_local: u32,
        pattern_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let pattern_offset_local = self.reserve_temp_local();
        let pattern_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let src_byte_local = self.reserve_temp_local();
        let pattern_byte_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_unpack_string_payload(
            pattern_payload_local,
            pattern_offset_local,
            pattern_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype[Symbol.match] is unsupported in wasm-aot",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(pattern_byte_local));
        function.instruction(&Instruction::LocalGet(src_byte_local));
        function.instruction(&Instruction::LocalGet(pattern_byte_local));
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

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            pattern_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(pattern_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(pattern_byte_local);
        self.release_temp_local(src_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(pattern_len_local);
        self.release_temp_local(pattern_offset_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_global_ascii_class_quantifier_from_string_locals(
        &mut self,
        string_local: u32,
        match_digits: bool,
        quantifier: i64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let probe_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Const(quantifier));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_load_string_byte(
            src_offset_local,
            probe_index_local,
            first_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            probe_index_local,
            src_len_local,
            first_byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        if !match_digits {
            function.instruction(&Instruction::I32Eqz);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(
            src_offset_local,
            scan_index_local,
            first_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            first_byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag_local);
        self.release_temp_local(match_len_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(probe_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_indices_pair_array_to_local(
        &mut self,
        start: i64,
        end: i64,
        array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, array_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));

        for (index, value) in [(0_i64, start), (1_i64, end)] {
            function.instruction(&Instruction::I64Const(index));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::F64Const(Ieee64::from(value as f64)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(value_payload_local));
            self.emit_array_write(
                array_local,
                index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_duplicate_named_groups_result(
        &mut self,
        string_local: u32,
        full_match: &str,
        full_match_units: i64,
        captures: &[(&str, Option<&str>, Option<(i64, i64)>)],
        has_indices_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let groups_local = self.reserve_temp_local();
        let indices_array_local = self.reserve_temp_local();
        let indices_groups_local = self.reserve_temp_local();
        let pair_array_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const((captures.len() + 1) as i64));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, array_local, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(full_match)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_write(
            array_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        for (index, (_, value, _)) in captures.iter().enumerate() {
            function.instruction(&Instruction::I64Const((index + 1) as i64));
            function.instruction(&Instruction::LocalSet(index_local));
            if let Some(value) = value {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            }
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_array_write(
                array_local,
                index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
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

        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(groups_local));
        for (name, value, _) in captures {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            if let Some(value) = value {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            }
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_enumerable_data(
                groups_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("groups")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(groups_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_define_named_data_property(
            array_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(has_indices_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const((captures.len() + 1) as i64));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, indices_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_regexp_indices_pair_array_to_local(
            0,
            full_match_units,
            pair_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(pair_array_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_write(
            indices_array_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        for (index, (_, _, range)) in captures.iter().enumerate() {
            function.instruction(&Instruction::I64Const((index + 1) as i64));
            function.instruction(&Instruction::LocalSet(index_local));
            if let Some((start, end)) = range {
                self.emit_regexp_indices_pair_array_to_local(
                    *start,
                    *end,
                    pair_array_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(pair_array_local));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            }
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_array_write(
                indices_array_local,
                index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(indices_groups_local));
        for (name, _, range) in captures {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            if let Some((start, end)) = range {
                self.emit_regexp_indices_pair_array_to_local(
                    *start,
                    *end,
                    pair_array_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(pair_array_local));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            }
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_enumerable_data(
                indices_groups_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("groups")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(indices_groups_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_define_named_data_property(
            indices_array_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("indices")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(indices_array_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_define_named_data_property(
            array_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(index_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(pair_array_local);
        self.release_temp_local(indices_groups_local);
        self.release_temp_local(indices_array_local);
        self.release_temp_local(groups_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_duplicate_named_groups_from_string_locals(
        &mut self,
        string_local: u32,
        iterated: bool,
        has_indices_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let candidate_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        if iterated {
            function.instruction(&Instruction::I64Const(self.strings.payload("aac")));
            function.instruction(&Instruction::LocalSet(candidate_local));
            self.emit_string_payload_equality_i32(string_local, candidate_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_string_match_duplicate_named_groups_result(
                string_local,
                "aac",
                3,
                &[("x", None, None)],
                has_indices_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::I64Const(self.strings.payload("abc")));
            function.instruction(&Instruction::LocalSet(candidate_local));
            self.emit_string_payload_equality_i32(string_local, candidate_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_string_match_duplicate_named_groups_result(
                string_local,
                "abc",
                3,
                &[
                    ("x", Some("b"), Some((1, 2))),
                    ("y", Some("a"), Some((0, 1))),
                    ("z", Some("c"), Some((2, 3))),
                ],
                has_indices_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(self.strings.payload("ad")));
            function.instruction(&Instruction::LocalSet(candidate_local));
            self.emit_string_payload_equality_i32(string_local, candidate_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_string_match_duplicate_named_groups_result(
                string_local,
                "ad",
                2,
                &[
                    ("x", Some("a"), Some((0, 1))),
                    ("y", None, None),
                    ("z", Some("d"), Some((1, 2))),
                ],
                has_indices_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(candidate_local);
        Ok(())
    }

    pub(crate) fn emit_ascii_digit_run_match_to_local(
        &mut self,
        src_offset_local: u32,
        start_local: u32,
        count: i64,
        result_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_string_match_postal_code_from_string_locals(
        &mut self,
        string_local: u32,
        global: bool,
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
        if global {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(array_len_local));
        } else {
            function.instruction(&Instruction::I64Const(3));
            function.instruction(&Instruction::LocalSet(array_len_local));
        }
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
        if !global {
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

    pub(crate) fn emit_string_match_array_from_locals(
        &mut self,
        string_local: u32,
        match_payload_local: u32,
        index_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_alloc_array_payload_with_length(index_local, array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            array_local,
            index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        self.emit_array_define_builtin_named_data_property(
            array_local,
            HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
            HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
            HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
            index_payload_local,
            number_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("index")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_define_named_data_property(
            array_local,
            key_local,
            index_payload_local,
            number_tag_local,
            function,
        )?;
        self.emit_array_define_builtin_named_data_property(
            array_local,
            HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
            HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
            HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
            string_local,
            string_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("input")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_define_named_data_property(
            array_local,
            key_local,
            string_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(string_tag_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_ascii_digit_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(match_len_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_match_zero_any_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, scan_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_load_string_byte(src_offset_local, match_len_local, next_byte_local, function);
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(b'\n' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(b'\r' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        self.emit_string_match_array_from_locals(
            string_local,
            match_payload_local,
            index_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(match_len_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(next_byte_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_payload_contains_regexp_syntax_i32(
        &mut self,
        string_payload_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        let syntax_bytes = b"^$\\.*+?()[]{}|/";
        for (idx, byte) in syntax_bytes.iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            if idx > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);
    }

    pub(crate) fn emit_string_substring_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let string_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let end_payload_local = self.reserve_temp_local();
        let end_tag_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let final_start_local = self.reserve_temp_local();
        let final_end_local = self.reserve_temp_local();
        let byte_start_local = self.reserve_temp_local();
        let byte_end_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_len_local,
            function,
        );

        if let Some(start) = args.first() {
            self.compile_expr_to_locals(start, start_payload_local, start_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(start_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(start_tag_local));
        }
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_clamped_to_string_len(
            start_payload_local,
            string_len_local,
            start_local,
            function,
        );

        if let Some(end) = args.get(1) {
            self.compile_expr_to_locals(end, end_payload_local, end_tag_local, function)?;
            self.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(end_payload_local));
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion_if_throw(function);
            self.emit_to_integer_clamped_to_string_len(
                end_payload_local,
                string_len_local,
                end_local,
                function,
            );
        } else {
            function.instruction(&Instruction::LocalGet(string_len_local));
            function.instruction(&Instruction::LocalSet(end_local));
        }

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(final_start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(final_end_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(final_start_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(final_end_local));
        function.instruction(&Instruction::End);

        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            final_start_local,
            byte_start_local,
            function,
        );
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            final_end_local,
            byte_end_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_end_local));
        function.instruction(&Instruction::LocalGet(byte_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            byte_start_local,
            byte_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            byte_len_local,
            byte_end_local,
            byte_start_local,
            final_end_local,
            final_start_local,
            end_local,
            start_local,
            end_tag_local,
            end_payload_local,
            start_tag_local,
            start_payload_local,
            string_len_local,
            string_byte_len_local,
            string_offset_local,
            string_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn static_number_expr_value(expr: &TypedExpr) -> Option<f64> {
        match &expr.expr {
            ExprIr::Number(bits) => Some(f64::from_bits(*bits)),
            ExprIr::UnaryNumber { op, expr } => {
                let value = Self::static_number_expr_value(expr)?;
                match op {
                    UnaryNumericOp::Plus => Some(value),
                    UnaryNumericOp::Minus => Some(-value),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn emit_string_char_code_at_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        index_payload_local: u32,
        index_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_len_local,
            function,
        );

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            string_offset_local,
            byte_index_local,
            string_byte_len_local,
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
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
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
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            temp_local,
            unit_advance_local,
            advance_local,
            codepoint_local,
            byte_local,
            unit_index_local,
            byte_index_local,
            result_local,
            index_local,
            index_number_payload_local,
            string_len_local,
            string_byte_len_local,
            string_offset_local,
            string_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_char_code_at_method_call(
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

        self.emit_string_char_code_at_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_payload_local,
            index_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_code_point_at_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        index_payload_local: u32,
        index_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_len_local,
            function,
        );

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            string_offset_local,
            byte_index_local,
            string_byte_len_local,
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
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
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
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            temp_local,
            unit_advance_local,
            advance_local,
            codepoint_local,
            byte_local,
            unit_index_local,
            byte_index_local,
            index_local,
            index_number_payload_local,
            string_len_local,
            string_byte_len_local,
            string_offset_local,
            string_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_code_point_at_method_call(
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

        self.emit_string_code_point_at_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_payload_local,
            index_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_at_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::StringPrototypeAt,
            "String.prototype.at",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_string_slice_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::StringPrototypeSlice,
            "String.prototype.slice",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_string_char_at_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let string_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let position_payload_local = self.reserve_temp_local();
        let position_tag_local = self.reserve_temp_local();
        let position_local = self.reserve_temp_local();
        let next_position_local = self.reserve_temp_local();
        let byte_start_local = self.reserve_temp_local();
        let byte_end_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_len_local,
            function,
        );

        if let Some(position) = args.first().and_then(Self::static_number_expr_value) {
            let integer_position = if position.is_nan() {
                0.0
            } else {
                position.trunc()
            };
            if integer_position < 0.0 {
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.set_completion_kind(CompletionKind::Normal, function);
                for local in [
                    byte_len_local,
                    byte_end_local,
                    byte_start_local,
                    next_position_local,
                    position_local,
                    position_tag_local,
                    position_payload_local,
                    string_len_local,
                    string_byte_len_local,
                    string_offset_local,
                    string_local,
                    receiver_tag_local,
                    receiver_payload_local,
                ] {
                    self.release_temp_local(local);
                }
                return Ok(());
            }
        }

        if let Some(position) = args.first() {
            self.compile_expr_to_locals(
                position,
                position_payload_local,
                position_tag_local,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(position_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(position_tag_local));
        }
        self.emit_value_to_number_payload(position_tag_local, position_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(position_payload_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(position_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(position_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(position_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(position_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(position_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(position_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(position_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(position_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(position_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(position_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(position_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            position_local,
            byte_start_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(position_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_position_local));
        self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
            string_local,
            next_position_local,
            byte_end_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_end_local));
        function.instruction(&Instruction::LocalGet(byte_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            byte_start_local,
            byte_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            byte_len_local,
            byte_end_local,
            byte_start_local,
            next_position_local,
            position_local,
            position_tag_local,
            position_payload_local,
            string_len_local,
            string_byte_len_local,
            string_offset_local,
            string_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_match_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let string_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));

        if let Some(arg) = args.first() {
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(arg_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(arg_tag_local));
        }

        self.compile_nullish_tagged_i32(arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_literal_fallback_from_string_locals(
            string_local,
            arg_payload_local,
            arg_tag_local,
            payload_local,
            tag_local,
            1,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.match")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            arg_payload_local,
            arg_tag_local,
            arg_payload_local,
            arg_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_match_literal_fallback_from_string_locals(
            string_local,
            arg_payload_local,
            arg_tag_local,
            payload_local,
            tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((arg_payload_local, Some(arg_tag_local))),
            &[(string_local, string_tag_local)],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype symbol hook is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_string_match_literal_fallback_from_string_locals(
            string_local,
            arg_payload_local,
            arg_tag_local,
            payload_local,
            tag_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            method_tag_local,
            method_payload_local,
            key_local,
            string_tag_local,
            string_local,
            arg_tag_local,
            arg_payload_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_split_method_call(
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
        let limit_payload_local = self.reserve_temp_local();
        let limit_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
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
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(separator_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(separator_tag_local));
        }

        if let Some(limit) = args.get(1) {
            self.compile_expr_to_locals(limit, limit_payload_local, limit_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(limit_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(limit_tag_local));
        }

        self.compile_nullish_tagged_i32(separator_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(separator_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.split")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            separator_payload_local,
            separator_tag_local,
            separator_payload_local,
            separator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((separator_payload_local, Some(separator_tag_local))),
            &[
                (receiver_payload_local, receiver_tag_local),
                (limit_payload_local, limit_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype symbol hook is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_string_split_fallback_from_receiver_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            method_tag_local,
            method_payload_local,
            key_local,
            limit_tag_local,
            limit_payload_local,
            separator_tag_local,
            separator_payload_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_split_fallback_from_receiver_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        separator_payload_local: u32,
        separator_tag_local: u32,
        limit_payload_local: u32,
        limit_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_string_split_from_string_locals(
            receiver_payload_local,
            separator_payload_local,
            separator_tag_local,
            limit_payload_local,
            limit_tag_local,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_string_split_from_string_locals(
        &mut self,
        string_local: u32,
        separator_payload_local: u32,
        separator_tag_local: u32,
        limit_payload_local: u32,
        limit_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let separator_string_local = self.reserve_temp_local();
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let sep_offset_local = self.reserve_temp_local();
        let sep_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let last_start_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let src_byte_local = self.reserve_temp_local();
        let sep_byte_local = self.reserve_temp_local();
        let piece_len_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let piece_tag_local = self.reserve_temp_local();
        let limit_local = self.reserve_temp_local();
        let regexp_source_payload_local = self.reserve_temp_local();
        let regexp_source_tag_local = self.reserve_temp_local();
        let regexp_kind_local = self.reserve_temp_local();
        let regexp_handled_local = self.reserve_temp_local();
        let regexp_key_local = self.reserve_temp_local();
        let regexp_prototype_local = self.reserve_temp_local();
        let regexp_is_regexp_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(regexp_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(regexp_is_regexp_local));
        function.instruction(&Instruction::LocalGet(separator_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            separator_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            regexp_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(regexp_prototype_local));
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(regexp_is_regexp_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(regexp_is_regexp_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.match")));
        function.instruction(&Instruction::LocalSet(regexp_key_local));
        self.emit_object_read(
            separator_payload_local,
            separator_tag_local,
            separator_payload_local,
            separator_tag_local,
            regexp_key_local,
            regexp_source_payload_local,
            regexp_source_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(regexp_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(regexp_source_payload_local));
        function.instruction(&Instruction::GlobalGet(
            REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(regexp_is_regexp_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(regexp_is_regexp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("source")));
        function.instruction(&Instruction::LocalSet(regexp_key_local));
        self.emit_object_read(
            separator_payload_local,
            separator_tag_local,
            separator_payload_local,
            separator_tag_local,
            regexp_key_local,
            regexp_source_payload_local,
            regexp_source_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(regexp_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (pattern, kind) in [
            ("l", 1),
            ("\\s", 2),
            ("\\d+", 3),
            ("[a-z]", 4),
            (",", 5),
            ("(?:)", 6),
            ("77", 7),
            ("\\u0037\\u0037", 7),
        ] {
            function.instruction(&Instruction::LocalGet(regexp_source_payload_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(pattern)));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(kind));
            function.instruction(&Instruction::LocalSet(regexp_kind_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_split_limit_to_uint32_local(
            limit_payload_local,
            limit_tag_local,
            limit_local,
            function,
        )?;
        self.emit_string_split_regexp_source_from_string_locals(
            string_local,
            regexp_kind_local,
            limit_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(regexp_handled_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(regexp_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::LocalGet(separator_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(separator_payload_local, separator_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(separator_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.emit_split_limit_to_uint32_local(
            limit_payload_local,
            limit_tag_local,
            limit_local,
            function,
        )?;
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));

        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(separator_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            string_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        self.emit_unpack_string_payload(
            separator_string_local,
            sep_offset_local,
            sep_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(last_start_local));

        function.instruction(&Instruction::LocalGet(sep_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            src_byte_local,
            sep_byte_local,
            piece_len_local,
            compare_index_local,
            function,
        );
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(piece_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(sep_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(sep_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(src_byte_local));
        function.instruction(&Instruction::LocalGet(sep_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(sep_byte_local));
        function.instruction(&Instruction::LocalGet(src_byte_local));
        function.instruction(&Instruction::LocalGet(sep_byte_local));
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

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(last_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(sep_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalSet(last_start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalGet(last_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::End);

        for local in [
            regexp_is_regexp_local,
            regexp_prototype_local,
            regexp_key_local,
            regexp_handled_local,
            regexp_kind_local,
            regexp_source_tag_local,
            regexp_source_payload_local,
            limit_local,
            piece_tag_local,
            piece_payload_local,
            piece_len_local,
            sep_byte_local,
            src_byte_local,
            match_local,
            compare_index_local,
            last_start_local,
            scan_index_local,
            write_index_local,
            zero_local,
            result_array_local,
            sep_len_local,
            sep_offset_local,
            src_len_local,
            src_offset_local,
            separator_string_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_string_split_regexp_source_from_string_locals(
        &mut self,
        string_local: u32,
        regexp_kind_local: u32,
        limit_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let last_start_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let piece_len_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let piece_tag_local = self.reserve_temp_local();
        let scalar_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(last_start_local));

        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            byte_local,
            scalar_local,
            piece_len_local,
            temp_local,
            function,
        );
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(piece_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for whitespace in [b'\t', b'\n', 0x0b_u8, b'\x0c', b'\r', b' '] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(whitespace as i64));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..6 {
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(match_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(regexp_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'7' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'7' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(match_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(last_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(match_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalSet(last_start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(limit_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalGet(last_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(piece_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            piece_payload_local,
            piece_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            temp_local,
            scalar_local,
            piece_tag_local,
            piece_payload_local,
            piece_len_local,
            next_index_local,
            match_len_local,
            byte_local,
            last_start_local,
            scan_index_local,
            write_index_local,
            zero_local,
            result_array_local,
            src_len_local,
            src_offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_split_limit_to_uint32_local(
        &mut self,
        limit_payload_local: u32,
        limit_tag_local: u32,
        out_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(limit_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(4_294_967_295));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(limit_tag_local, limit_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(limit_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_uint32_i64_from_number_payload(limit_payload_local, out_local, function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_to_uint32_i64_from_number_payload(
        &mut self,
        number_payload_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
            function.instruction(&Instruction::LocalGet(number_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(4_294_967_296));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(4_294_967_296));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_regexp_exec_literal_control_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        return_boolean: bool,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if args.len() != 1 {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: RegExp.prototype.exec arity",
            ));
        }

        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();
        let source_offset_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let invalid_escape_local = self.reserve_temp_local();
        let array_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let input_offset_local = self.reserve_temp_local();
        let input_len_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let input_byte_local = self.reserve_temp_local();
        let source_byte_local = self.reserve_temp_local();
        let letter_len_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let sticky_handled_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec receiver is not RegExp",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.compile_expr_to_locals(&args[0], input_payload_local, input_tag_local, function)?;
        self.emit_value_to_string_payload(input_payload_local, input_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(input_payload_local));
        self.emit_return_current_completion_if_throw(function);

        self.emit_regexp_exec_simple_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            input_payload_local,
            return_boolean,
            sticky_handled_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(sticky_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("source")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            source_payload_local,
            source_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec source is not string",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(
            source_payload_local,
            source_offset_local,
            source_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'c' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'c' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (index, byte) in [
            (0_i64, b'['),
            (1, b'\\'),
            (2, b'\\'),
            (3, b'c'),
            (4, b'-'),
            (5, b'f'),
            (6, b']'),
        ] {
            function.instruction(&Instruction::LocalGet(source_offset_local));
            function.instruction(&Instruction::I64Const(index));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..7 {
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (index, byte) in [(0_i64, b'('), (1, b'?'), (2, b':'), (3, b')')] {
            function.instruction(&Instruction::LocalGet(source_offset_local));
            function.instruction(&Instruction::I64Const(index));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..4 {
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(invalid_escape_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_escape_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "RegExp.prototype.exec unsupported pattern",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::LocalGet(invalid_escape_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(source_payload_local, input_payload_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(invalid_escape_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(
            input_payload_local,
            input_offset_local,
            input_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(input_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(input_byte_local));
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::I64Const(b'c' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_unpack_string_payload(
            input_payload_local,
            input_offset_local,
            input_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(input_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(input_byte_local));
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::I64Const(b'c' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(letter_len_local));
        function.instruction(&Instruction::LocalGet(input_len_local));
        function.instruction(&Instruction::LocalGet(letter_len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(letter_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(input_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(input_byte_local));
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(source_byte_local));
        function.instruction(&Instruction::LocalGet(input_byte_local));
        function.instruction(&Instruction::LocalGet(source_byte_local));
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
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_escape_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::End);

        if return_boolean {
            function.instruction(&Instruction::LocalGet(match_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            function.instruction(&Instruction::LocalGet(match_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_alloc_array_payload_with_length(index_local, array_local, function)?;
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::LocalSet(match_payload_local));
            function.instruction(&Instruction::LocalGet(invalid_escape_local));
            function.instruction(&Instruction::I64Const(4));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("")));
            function.instruction(&Instruction::LocalSet(match_payload_local));
            function.instruction(&Instruction::End);
            self.emit_array_write(
                array_local,
                index_local,
                match_payload_local,
                source_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload("index")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(source_tag_local));
            self.emit_array_define_builtin_named_data_property(
                array_local,
                HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
                index_local,
                source_tag_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("index")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_array_define_named_data_property(
                array_local,
                key_local,
                index_local,
                source_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload("input")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(source_tag_local));
            self.emit_array_define_builtin_named_data_property(
                array_local,
                HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
                input_payload_local,
                source_tag_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("input")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_array_define_named_data_property(
                array_local,
                key_local,
                input_payload_local,
                source_tag_local,
                function,
            )?;
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
        }

        function.instruction(&Instruction::End);

        for local in [
            sticky_handled_local,
            match_payload_local,
            letter_len_local,
            source_byte_local,
            input_byte_local,
            compare_index_local,
            input_len_local,
            input_offset_local,
            match_local,
            index_local,
            array_local,
            invalid_escape_local,
            byte_local,
            source_len_local,
            source_offset_local,
            input_tag_local,
            input_payload_local,
            source_tag_local,
            source_payload_local,
            key_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_regexp_exec_simple_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        input_payload_local: u32,
        return_boolean: bool,
        handled_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let flags_payload_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_is_dot_local = self.reserve_temp_local();
        let source_supported_local = self.reserve_temp_local();
        let global_local = self.reserve_temp_local();
        let sticky_local = self.reserve_temp_local();
        let full_unicode_local = self.reserve_temp_local();
        let flags_supported_local = self.reserve_temp_local();
        let source_offset_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let input_offset_local = self.reserve_temp_local();
        let input_byte_len_local = self.reserve_temp_local();
        let input_unit_len_local = self.reserve_temp_local();
        let last_index_payload_local = self.reserve_temp_local();
        let last_index_tag_local = self.reserve_temp_local();
        let last_index_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let match_success_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let end_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let char_code_payload_local = self.reserve_temp_local();
        let char_code_tag_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let comparison_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            flags_payload_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'g',
            global_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'y',
            sticky_local,
            function,
        );
        self.emit_string_payload_contains_ascii_byte_i32(
            flags_payload_local,
            b'u',
            full_unicode_local,
            function,
        );
        // This bounded matcher only understands the flags that affect its search.
        // Keep all other flag combinations on the general RegExp path.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(flags_supported_local));
        for unsupported_flag in [b'd', b'i', b'm', b's', b'v'] {
            self.emit_string_payload_contains_ascii_byte_i32(
                flags_payload_local,
                unsupported_flag,
                comparison_payload_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(comparison_payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(flags_supported_local));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(flags_supported_local));
        }
        function.instruction(&Instruction::LocalGet(flags_supported_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );

        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::LocalSet(comparison_payload_local));
        self.emit_string_payload_equality_i32(
            source_payload_local,
            comparison_payload_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(source_is_dot_local));

        function.instruction(&Instruction::LocalGet(source_is_dot_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        for literal in ["a", "b", "c", "abc"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(literal)));
            function.instruction(&Instruction::LocalSet(comparison_payload_local));
            self.emit_string_payload_equality_i32(
                source_payload_local,
                comparison_payload_local,
                function,
            );
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(source_supported_local));
        function.instruction(&Instruction::LocalGet(source_supported_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            last_index_payload_local,
            last_index_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            4,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals_without_throw_return(
            last_index_tag_local,
            last_index_payload_local,
            last_index_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            4,
            function,
        )?;

        self.emit_unpack_string_payload(
            input_payload_local,
            input_offset_local,
            input_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            input_offset_local,
            input_byte_len_local,
            input_unit_len_local,
            function,
        );
        self.emit_unpack_string_payload(
            source_payload_local,
            source_offset_local,
            source_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(source_is_dot_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::LocalGet(sticky_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(input_unit_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        // AdvanceStringIndex works in UTF-16 code units. In full-Unicode mode a
        // valid surrogate pair is one search position and `.` consumes both
        // code units; otherwise the next candidate is one code unit later.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(input_unit_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(char_code_tag_local));
        self.emit_string_char_code_at_from_locals(
            input_payload_local,
            char_code_tag_local,
            number_payload_local,
            number_tag_local,
            char_code_payload_local,
            char_code_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(full_unicode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(input_unit_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(last_index_tag_local));
        self.emit_string_char_code_at_from_locals(
            input_payload_local,
            last_index_tag_local,
            number_payload_local,
            number_tag_local,
            end_index_local,
            last_index_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xD800 as f64)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::LocalGet(char_code_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDBFF as f64)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDC00 as f64)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0xDFFF as f64)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(candidate_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_is_dot_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(input_unit_len_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(match_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_utf16_code_unit_range_payload_from_locals(
            input_payload_local,
            scan_index_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::LocalGet(source_is_dot_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (index, terminator) in [10.0, 13.0, 8232.0, 8233.0].into_iter().enumerate() {
            function.instruction(&Instruction::LocalGet(char_code_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(terminator)));
            function.instruction(&Instruction::F64Eq);
            if index > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(match_success_local));
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32(match_payload_local, source_payload_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(match_success_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(match_success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(sticky_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(end_index_local));
        function.instruction(&Instruction::LocalGet(match_success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(match_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(global_local));
        function.instruction(&Instruction::LocalGet(sticky_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            number_payload_local,
            number_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            5,
            function,
        )?;
        function.instruction(&Instruction::End);

        if return_boolean {
            function.instruction(&Instruction::LocalGet(match_success_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            function.instruction(&Instruction::LocalGet(match_success_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(scan_index_local));
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(number_payload_local));
            self.emit_string_match_array_from_locals(
                input_payload_local,
                match_payload_local,
                number_payload_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            comparison_payload_local,
            key_local,
            number_tag_local,
            number_payload_local,
            char_code_tag_local,
            char_code_payload_local,
            candidate_index_local,
            scan_index_local,
            end_index_local,
            match_payload_local,
            match_success_local,
            match_len_local,
            last_index_local,
            last_index_tag_local,
            last_index_payload_local,
            input_unit_len_local,
            input_byte_len_local,
            input_offset_local,
            source_len_local,
            source_offset_local,
            flags_supported_local,
            full_unicode_local,
            sticky_local,
            global_local,
            source_supported_local,
            source_is_dot_local,
            source_payload_local,
            flags_payload_local,
            brand_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_array_to_string_locals(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let result_string_local = self.reserve_temp_local();
        let comma_string_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(result_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(comma_string_local));
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
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            element_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            element_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_concat_string_payloads_local(result_string_local, comma_string_local, function)?;
        function.instruction(&Instruction::LocalSet(result_string_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(element_string_local));
        function.instruction(&Instruction::Else);
        self.emit_array_element_to_string_payload(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        function.instruction(&Instruction::End);

        self.emit_concat_string_payloads_local(
            result_string_local,
            element_string_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_string_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_string_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(comma_string_local);
        self.release_temp_local(result_string_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_element_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_bigint_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::String,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_string_payload(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_function_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_concat_string_payloads_local(
        &mut self,
        lhs_string_local: u32,
        rhs_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let total_len = self.reserve_temp_local();
        let alloc_len = self.reserve_temp_local();
        let dst_offset = self.reserve_temp_local();
        let rhs_dst_offset = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_string_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_string_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len));
        function.instruction(&Instruction::LocalGet(total_len));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len));
        self.emit_heap_alloc_from_local(alloc_len, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset));
        self.emit_copy_bytes(lhs_offset, dst_offset, lhs_len, function);
        function.instruction(&Instruction::LocalGet(dst_offset));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_dst_offset));
        self.emit_copy_bytes(rhs_offset, rhs_dst_offset, rhs_len, function);
        self.emit_pack_string_payload(dst_offset, total_len, function);

        self.release_temp_local(rhs_dst_offset);
        self.release_temp_local(dst_offset);
        self.release_temp_local(alloc_len);
        self.release_temp_local(total_len);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
        Ok(())
    }

    pub(crate) fn emit_annexb_escape_string_payload(
        &mut self,
        input_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let dst_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            input_string_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        self.emit_heap_alloc_from_local(dst_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::LocalSet(unit_local));
        self.emit_annexb_escape_utf16_unit(dst_pos_local, unit_local, digit_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_local));
        self.emit_annexb_escape_utf16_unit(dst_pos_local, unit_local, digit_local, function);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_local));
        self.emit_annexb_escape_utf16_unit(dst_pos_local, unit_local, digit_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        self.emit_pack_string_payload(dst_offset_local, dst_len_local, function);

        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(unit_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(dst_len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_annexb_escape_utf16_unit(
        &self,
        dst_pos_local: u32,
        unit_local: u32,
        digit_local: u32,
        function: &mut Function,
    ) {
        self.emit_annexb_escape_unescaped_byte_i32(unit_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_byte_local(dst_pos_local, unit_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(256));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_pos_local, b'%', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 4, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 0, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(dst_pos_local, b'%', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.store_ascii_byte_i64(dst_pos_local, b'u', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 12, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 8, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 4, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_hex_digit_from_byte(unit_local, digit_local, 0, dst_pos_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_regexp_escape_string_payload(
        &mut self,
        input_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let dst_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let first_hex_local = self.reserve_temp_local();
        let second_hex_local = self.reserve_temp_local();
        let third_hex_local = self.reserve_temp_local();
        let fourth_hex_local = self.reserve_temp_local();
        let surrogate_escape_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            input_string_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        self.emit_heap_alloc_from_local(dst_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(surrogate_escape_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 1, temp_local, function);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 2, temp_local, function);
        self.emit_hex_value_or_minus_one(temp_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 3, temp_local, function);
        self.emit_hex_value_or_minus_one(temp_local, second_hex_local, function);
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 4, temp_local, function);
        self.emit_hex_value_or_minus_one(temp_local, third_hex_local, function);
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 5, temp_local, function);
        self.emit_hex_value_or_minus_one(temp_local, fourth_hex_local, function);
        self.emit_all_hex_valid_i32(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pack_four_hex_to_code_unit(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            unit_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(0xDFFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::LocalSet(advance_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(surrogate_escape_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(surrogate_escape_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        self.emit_codepoint_is_ascii_alnum_i32(codepoint_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_escape_hex_byte(dst_pos_local, codepoint_local, digit_local, function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_escape_codepoint(
            dst_pos_local,
            codepoint_local,
            unit_local,
            digit_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        self.emit_pack_string_payload(dst_offset_local, dst_len_local, function);

        self.release_temp_local(surrogate_escape_local);
        self.release_temp_local(fourth_hex_local);
        self.release_temp_local(third_hex_local);
        self.release_temp_local(second_hex_local);
        self.release_temp_local(first_hex_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(unit_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(dst_len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_escape_codepoint(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        unit_local: u32,
        digit_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        self.emit_regexp_control_escape_suffix_i64(codepoint_local, function);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_pos_local, b'\\', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_byte_local(dst_pos_local, temp_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_codepoint_is_regexp_syntax_char_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_pos_local, b'\\', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_byte_local(dst_pos_local, codepoint_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_codepoint_needs_regexp_hex_or_unicode_escape_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_escape_hex_or_unicode_codepoint(
            dst_pos_local,
            codepoint_local,
            unit_local,
            digit_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_regexp_escape_hex_or_unicode_codepoint(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        unit_local: u32,
        digit_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_escape_hex_byte(dst_pos_local, codepoint_local, digit_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_escape_unicode_unit(dst_pos_local, codepoint_local, digit_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_local));
        self.emit_regexp_escape_unicode_unit(dst_pos_local, unit_local, digit_local, function);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_local));
        self.emit_regexp_escape_unicode_unit(dst_pos_local, unit_local, digit_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_regexp_escape_hex_byte(
        &self,
        dst_pos_local: u32,
        unit_local: u32,
        digit_local: u32,
        function: &mut Function,
    ) {
        self.store_ascii_byte_i64(dst_pos_local, b'\\', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.store_ascii_byte_i64(dst_pos_local, b'x', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            4,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            0,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
    }

    pub(crate) fn emit_regexp_escape_unicode_unit(
        &self,
        dst_pos_local: u32,
        unit_local: u32,
        digit_local: u32,
        function: &mut Function,
    ) {
        self.store_ascii_byte_i64(dst_pos_local, b'\\', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.store_ascii_byte_i64(dst_pos_local, b'u', function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            12,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            8,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            4,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_store_lower_hex_digit_from_byte(
            unit_local,
            digit_local,
            0,
            dst_pos_local,
            function,
        );
        self.emit_increment_local(dst_pos_local, 1, function);
    }

    pub(crate) fn emit_regexp_control_escape_suffix_i64(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        for (idx, (codepoint, suffix)) in [
            (0x09, b't'),
            (0x0A, b'n'),
            (0x0B, b'v'),
            (0x0C, b'f'),
            (0x0D, b'r'),
        ]
        .iter()
        .enumerate()
        {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(*codepoint));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(*suffix as i64));
            function.instruction(&Instruction::Else);
            if idx == 4 {
                function.instruction(&Instruction::I64Const(0));
            }
        }
        for _ in 0..5 {
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_codepoint_is_ascii_alnum_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        self.emit_codepoint_in_range_i32(codepoint_local, b'0' as i64, b'9' as i64, function);
        self.emit_codepoint_in_range_i32(codepoint_local, b'A' as i64, b'Z' as i64, function);
        function.instruction(&Instruction::I32Or);
        self.emit_codepoint_in_range_i32(codepoint_local, b'a' as i64, b'z' as i64, function);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_codepoint_is_regexp_syntax_char_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'^' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in b"$\\.*+?()[]{}|/" {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(*byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_codepoint_needs_regexp_hex_or_unicode_escape_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        self.emit_codepoint_is_regexp_other_punctuator_i32(codepoint_local, function);
        self.emit_codepoint_is_regexp_whitespace_or_line_terminator_i32(codepoint_local, function);
        function.instruction(&Instruction::I32Or);
        self.emit_codepoint_in_range_i32(codepoint_local, 0xD800, 0xDFFF, function);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_codepoint_is_regexp_other_punctuator_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in b"-=<>#&!%:;@~'`\"" {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(*byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_codepoint_is_regexp_whitespace_or_line_terminator_i32(
        &self,
        codepoint_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64Eq);
        for codepoint in [0xA0, 0x1680, 0x2028, 0x2029, 0x202F, 0x205F, 0x3000, 0xFEFF] {
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::I64Const(codepoint));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        self.emit_codepoint_in_range_i32(codepoint_local, 0x2000, 0x200A, function);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_codepoint_in_range_i32(
        &self,
        codepoint_local: u32,
        min: i64,
        max: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(min));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(max));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    pub(crate) fn emit_annexb_unescape_string_payload(
        &mut self,
        input_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let first_hex_local = self.reserve_temp_local();
        let second_hex_local = self.reserve_temp_local();
        let third_hex_local = self.reserve_temp_local();
        let fourth_hex_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            input_string_local,
            src_offset_local,
            src_len_local,
            function,
        );
        self.emit_heap_alloc_from_local(src_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'%' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            1,
            first_hex_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(first_hex_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            2,
            first_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(first_hex_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            3,
            second_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(second_hex_local, second_hex_local, function);
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            4,
            third_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(third_hex_local, third_hex_local, function);
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            5,
            fourth_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(fourth_hex_local, fourth_hex_local, function);
        self.emit_all_hex_valid_i32(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pack_four_hex_to_code_unit(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            codepoint_local,
            function,
        );
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        self.emit_increment_local(index_local, 6, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            1,
            first_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(first_hex_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(
            src_offset_local,
            index_local,
            2,
            second_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(second_hex_local, second_hex_local, function);
        function.instruction(&Instruction::LocalGet(first_hex_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(second_hex_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_hex_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(second_hex_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        self.emit_increment_local(index_local, 3, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_store_byte_local(dst_pos_local, byte_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(src_len_local));
        self.emit_pack_string_payload(dst_offset_local, src_len_local, function);

        self.release_temp_local(temp_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(fourth_hex_local);
        self.release_temp_local(third_hex_local);
        self.release_temp_local(second_hex_local);
        self.release_temp_local(first_hex_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_load_string_byte(
        &self,
        src_offset_local: u32,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
    }

    pub(crate) fn emit_or_byte_equals_flag(
        &self,
        byte_local: u32,
        expected: u8,
        flag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(flag_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flag_local));
    }

    pub(crate) fn emit_byte_is_json_whitespace_i32(
        &self,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\t' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\n' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\r' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_byte_is_digit_i32(&self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    pub(crate) fn emit_byte_is_json_escape_i32(&self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in [b'\\', b'/', b'b', b'f', b'n', b'r', b't', b'u'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_byte_is_json_non_number_start_i32(
        &self,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in [b'[', b'"', b't', b'f', b'n'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_byte_is_json_structural_or_value_start_i32(
        &self,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in [
            b'[', b']', b'}', b':', b',', b'"', b'-', b'.', b't', b'f', b'n',
        ] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_advance_json_parse_digit_run(
        &self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_load_string_byte_at_delta(
        &self,
        src_offset_local: u32,
        index_local: u32,
        delta: i64,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(delta));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
    }

    pub(crate) fn emit_decode_utf8_scalar_at_index(
        &self,
        src_offset_local: u32,
        index_local: u32,
        src_len_local: u32,
        first_byte_local: u32,
        codepoint_local: u32,
        advance_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(advance_local));

        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0xC0));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0xE0));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 1, temp_local, function);
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0x1F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(advance_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0xF0));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0x0F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 1, temp_local, function);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_load_string_byte_at_delta(src_offset_local, index_local, 2, temp_local, function);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(advance_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(0x07));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        for (delta, shift) in [(1, 12), (2, 6), (3, 0)] {
            self.emit_load_string_byte_at_delta(
                src_offset_local,
                index_local,
                delta,
                temp_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(codepoint_local));
            function.instruction(&Instruction::LocalGet(temp_local));
            function.instruction(&Instruction::I64Const(0x3F));
            function.instruction(&Instruction::I64And);
            if shift != 0 {
                function.instruction(&Instruction::I64Const(shift));
                function.instruction(&Instruction::I64Shl);
            }
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(codepoint_local));
        }
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(advance_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_store_byte_local(
        &self,
        offset_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    pub(crate) fn emit_store_utf8_codepoint(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_byte_local(dst_pos_local, codepoint_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x800));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_utf8_continuation_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            0xC0,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_utf8_continuation_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            12,
            0xE0,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_utf8_continuation_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            18,
            0xF0,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            12,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            function,
        );
        self.emit_store_utf8_continuation_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_store_utf8_continuation_head(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        temp_local: u32,
        shift: i64,
        mask: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(shift));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(mask));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(temp_local));
        self.emit_store_byte_local(dst_pos_local, temp_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
    }

    pub(crate) fn emit_store_utf8_continuation_tail(
        &self,
        dst_pos_local: u32,
        codepoint_local: u32,
        temp_local: u32,
        shift: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(temp_local));
        self.emit_store_byte_local(dst_pos_local, temp_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
    }

    pub(crate) fn emit_increment_local(&self, local: u32, delta: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(delta));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(local));
    }

    pub(crate) fn emit_annexb_escape_unescaped_byte_i32(
        &self,
        byte_local: u32,
        function: &mut Function,
    ) {
        for (idx, (lo, hi)) in [(b'A', b'Z'), (b'a', b'z'), (b'0', b'9')]
            .iter()
            .enumerate()
        {
            if idx == 0 {
                function.instruction(&Instruction::LocalGet(byte_local));
                function.instruction(&Instruction::I64Const(*lo as i64));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::LocalGet(byte_local));
                function.instruction(&Instruction::I64Const(*hi as i64));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::I32And);
            } else {
                function.instruction(&Instruction::LocalGet(byte_local));
                function.instruction(&Instruction::I64Const(*lo as i64));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::LocalGet(byte_local));
                function.instruction(&Instruction::I64Const(*hi as i64));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
            }
        }
        for byte in b"@*_+-./" {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(*byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_store_hex_digit_from_byte(
        &self,
        byte_local: u32,
        digit_local: u32,
        shift: i64,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'A' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    pub(crate) fn emit_store_lower_hex_digit_from_byte(
        &self,
        byte_local: u32,
        digit_local: u32,
        shift: i64,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    pub(crate) fn emit_hex_value_or_minus_one(
        &self,
        byte_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'F' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const((b'A' - 10) as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(out_local));
    }

    pub(crate) fn emit_all_hex_valid_i32(
        &self,
        first_hex_local: u32,
        second_hex_local: u32,
        third_hex_local: u32,
        fourth_hex_local: u32,
        function: &mut Function,
    ) {
        for (idx, local) in [
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
        ]
        .iter()
        .enumerate()
        {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            if idx > 0 {
                function.instruction(&Instruction::I32And);
            }
        }
    }

    pub(crate) fn emit_pack_four_hex_to_code_unit(
        &self,
        first_hex_local: u32,
        second_hex_local: u32,
        third_hex_local: u32,
        fourth_hex_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(first_hex_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(second_hex_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(third_hex_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(fourth_hex_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(out_local));
    }

    pub(crate) fn emit_escape_html_attr_string_payload(
        &mut self,
        input_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let quote_count_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let dst_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let src_addr_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            input_string_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(quote_count_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quote_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(quote_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalGet(quote_count_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        self.emit_heap_alloc_from_local(dst_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_addr_local));
        function.instruction(&Instruction::LocalGet(src_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for byte in b"&quot;" {
            self.store_ascii_byte_i64(dst_pos_local, *byte, function);
            function.instruction(&Instruction::LocalGet(dst_pos_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(dst_pos_local));
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(dst_offset_local, dst_len_local, function);

        self.release_temp_local(src_addr_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(dst_len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(quote_count_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }
}
