use super::*;

impl<'a> FunctionBuilder<'a> {
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
