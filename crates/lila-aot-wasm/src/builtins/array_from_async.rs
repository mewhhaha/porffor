use super::super::*;

const ARRAY_FROM_ASYNC_STATE_SIZE: u64 = 184;
const ARRAY_FROM_ASYNC_CAPABILITY_OFFSET: u64 = 0;
const ARRAY_FROM_ASYNC_THROWAWAY_CAPABILITY_OFFSET: u64 = 8;
const ARRAY_FROM_ASYNC_SOURCE_PAYLOAD_OFFSET: u64 = 16;
const ARRAY_FROM_ASYNC_SOURCE_TAG_OFFSET: u64 = 24;
const ARRAY_FROM_ASYNC_TARGET_PAYLOAD_OFFSET: u64 = 32;
const ARRAY_FROM_ASYNC_TARGET_TAG_OFFSET: u64 = 40;
const ARRAY_FROM_ASYNC_MAPPER_PAYLOAD_OFFSET: u64 = 48;
const ARRAY_FROM_ASYNC_MAPPER_TAG_OFFSET: u64 = 56;
const ARRAY_FROM_ASYNC_THIS_ARG_PAYLOAD_OFFSET: u64 = 64;
const ARRAY_FROM_ASYNC_THIS_ARG_TAG_OFFSET: u64 = 72;
const ARRAY_FROM_ASYNC_INDEX_OFFSET: u64 = 80;
const ARRAY_FROM_ASYNC_LENGTH_OFFSET: u64 = 88;
const ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET: u64 = 96;
const ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET: u64 = 104;
const ARRAY_FROM_ASYNC_STAGE_OFFSET: u64 = 112;
const ARRAY_FROM_ASYNC_ITERATOR_PAYLOAD_OFFSET: u64 = 120;
const ARRAY_FROM_ASYNC_ITERATOR_TAG_OFFSET: u64 = 128;
const ARRAY_FROM_ASYNC_NEXT_PAYLOAD_OFFSET: u64 = 136;
const ARRAY_FROM_ASYNC_NEXT_TAG_OFFSET: u64 = 144;
const ARRAY_FROM_ASYNC_MODE_OFFSET: u64 = 152;
const ARRAY_FROM_ASYNC_SAVED_ERROR_PAYLOAD_OFFSET: u64 = 160;
const ARRAY_FROM_ASYNC_SAVED_ERROR_TAG_OFFSET: u64 = 168;
const ARRAY_FROM_ASYNC_REALM_ENV_OFFSET: u64 = 176;

const ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE: u64 = 0;
const ARRAY_FROM_ASYNC_STAGE_MAPPED_VALUE: u64 = 1;
const ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT: u64 = 2;
const ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE: u64 = 3;
const ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT: u64 = 4;
const ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE: u64 = 5;

const ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE: u64 = 0;
const ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR: u64 = 1;
const ARRAY_FROM_ASYNC_MODE_SYNC_ITERATOR: u64 = 2;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_array_from_async(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.fromAsync receiver",
            )
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.fromAsync receiver tag",
            )
        })?;
        let promise_constructor_payload_local = self.reserve_temp_local();
        let promise_constructor_tag_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_object_payload_local = self.reserve_temp_local();
        let source_object_tag_local = self.reserve_temp_local();
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_mode_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(promise_constructor_tag_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);
        self.emit_builtin_arg_to_locals(1, mapper_payload_local, mapper_tag_local, function);
        self.emit_builtin_arg_to_locals(2, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::LocalGet(mapper_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_callable_i32(mapper_tag_local, mapper_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync input is null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            source_payload_local,
            source_tag_local,
            source_object_payload_local,
            source_object_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::LocalSet(iterator_mode_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.asyncIterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_object_read_without_throw_propagation_with_key_tag(
            source_object_payload_local,
            source_object_tag_local,
            source_payload_local,
            source_tag_local,
            key_local,
            key_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_SYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::LocalSet(iterator_mode_local));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation_with_key_tag(
            source_object_payload_local,
            source_object_tag_local,
            source_payload_local,
            source_tag_local,
            key_local,
            key_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_array_like_start(
            constructor_payload_local,
            constructor_tag_local,
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            source_object_payload_local,
            source_object_tag_local,
            mapper_payload_local,
            mapper_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_iterable_start(
            constructor_payload_local,
            constructor_tag_local,
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            source_payload_local,
            source_tag_local,
            method_payload_local,
            method_tag_local,
            iterator_mode_local,
            mapper_payload_local,
            mapper_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(iterator_mode_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        self.release_temp_local(source_object_tag_local);
        self.release_temp_local(source_object_payload_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(promise_tag_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(capability_record_local);
        self.release_temp_local(promise_constructor_tag_local);
        self.release_temp_local(promise_constructor_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_from_async_array_like_start(
        &mut self,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        promise_constructor_payload_local: u32,
        promise_constructor_tag_local: u32,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        source_payload_local: u32,
        source_tag_local: u32,
        mapper_payload_local: u32,
        mapper_tag_local: u32,
        this_arg_payload_local: u32,
        this_arg_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let is_constructor_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let throwaway_capability_local = self.reserve_temp_local();
        let throwaway_promise_payload_local = self.reserve_temp_local();
        let throwaway_promise_tag_local = self.reserve_temp_local();
        let fulfilled_callback_payload_local = self.reserve_temp_local();
        let rejected_callback_payload_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals_without_throw_return(
            length_tag_local,
            length_payload_local,
            length_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        self.emit_is_constructor_i32(constructor_tag_local, constructor_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_constructor_local));
        function.instruction(&Instruction::LocalGet(is_constructor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(length_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(length_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[(length_payload_local, length_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_set_length(
            target_payload_local,
            target_tag_local,
            length_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_array_from_async_resolve(
            capability_record_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_const(ARRAY_FROM_ASYNC_STATE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            throwaway_capability_local,
            throwaway_promise_payload_local,
            throwaway_promise_tag_local,
            function,
        )?;

        let fulfilled_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFromAsyncFulfilled.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Array.fromAsync fulfillment callback builtin")
            })?;
        self.emit_function_value_payload(&fulfilled_meta, function)?;
        function.instruction(&Instruction::LocalSet(fulfilled_callback_payload_local));
        self.store_i64_local_at_offset(
            fulfilled_callback_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            state_local,
            function,
        );

        let rejected_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFromAsyncRejected.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Array.fromAsync rejection callback builtin")
            })?;
        self.emit_function_value_payload(&rejected_meta, function)?;
        function.instruction(&Instruction::LocalSet(rejected_callback_payload_local));
        self.store_i64_local_at_offset(
            rejected_callback_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            state_local,
            function,
        );

        for (offset, local) in [
            (ARRAY_FROM_ASYNC_CAPABILITY_OFFSET, capability_record_local),
            (
                ARRAY_FROM_ASYNC_THROWAWAY_CAPABILITY_OFFSET,
                throwaway_capability_local,
            ),
            (ARRAY_FROM_ASYNC_SOURCE_PAYLOAD_OFFSET, source_payload_local),
            (ARRAY_FROM_ASYNC_SOURCE_TAG_OFFSET, source_tag_local),
            (ARRAY_FROM_ASYNC_TARGET_PAYLOAD_OFFSET, target_payload_local),
            (ARRAY_FROM_ASYNC_TARGET_TAG_OFFSET, target_tag_local),
            (ARRAY_FROM_ASYNC_MAPPER_PAYLOAD_OFFSET, mapper_payload_local),
            (ARRAY_FROM_ASYNC_MAPPER_TAG_OFFSET, mapper_tag_local),
            (
                ARRAY_FROM_ASYNC_THIS_ARG_PAYLOAD_OFFSET,
                this_arg_payload_local,
            ),
            (ARRAY_FROM_ASYNC_THIS_ARG_TAG_OFFSET, this_arg_tag_local),
            (ARRAY_FROM_ASYNC_LENGTH_OFFSET, length_local),
            (
                ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET,
                fulfilled_callback_payload_local,
            ),
            (
                ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET,
                rejected_callback_payload_local,
            ),
            (ARRAY_FROM_ASYNC_REALM_ENV_OFFSET, self.current_env_local),
        ] {
            self.store_i64_local_at_offset(state_local, offset, local, function);
        }
        self.store_i64_const_at_offset(state_local, ARRAY_FROM_ASYNC_INDEX_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE,
            function,
        );
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_MODE_OFFSET,
            ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE,
            function,
        );

        self.emit_array_from_async_read_array_like_value(
            state_local,
            input_payload_local,
            input_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_array_from_async_schedule_await(
            state_local,
            input_payload_local,
            input_tag_local,
            function,
        )?;

        self.release_temp_local(input_tag_local);
        self.release_temp_local(input_payload_local);
        self.release_temp_local(rejected_callback_payload_local);
        self.release_temp_local(fulfilled_callback_payload_local);
        self.release_temp_local(throwaway_promise_tag_local);
        self.release_temp_local(throwaway_promise_payload_local);
        self.release_temp_local(throwaway_capability_local);
        self.release_temp_local(state_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(is_constructor_local);
        self.release_temp_local(length_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_from_async_iterable_start(
        &mut self,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        promise_constructor_payload_local: u32,
        promise_constructor_tag_local: u32,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        source_payload_local: u32,
        source_tag_local: u32,
        iterator_method_payload_local: u32,
        iterator_method_tag_local: u32,
        iterator_mode_local: u32,
        mapper_payload_local: u32,
        mapper_tag_local: u32,
        this_arg_payload_local: u32,
        this_arg_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let is_constructor_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let throwaway_capability_local = self.reserve_temp_local();
        let throwaway_promise_payload_local = self.reserve_temp_local();
        let throwaway_promise_tag_local = self.reserve_temp_local();
        let fulfilled_callback_payload_local = self.reserve_temp_local();
        let rejected_callback_payload_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();

        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            source_payload_local,
            source_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_is_constructor_i32(constructor_tag_local, constructor_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_constructor_local));
        function.instruction(&Instruction::LocalGet(is_constructor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_array_payload_with_length(zero_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_pre_evaluated_arg_vector(&[], argc_local, argv_local, function)?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_const(ARRAY_FROM_ASYNC_STATE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            throwaway_capability_local,
            throwaway_promise_payload_local,
            throwaway_promise_tag_local,
            function,
        )?;

        let fulfilled_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFromAsyncFulfilled.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Array.fromAsync fulfillment callback builtin")
            })?;
        self.emit_function_value_payload(&fulfilled_meta, function)?;
        function.instruction(&Instruction::LocalSet(fulfilled_callback_payload_local));
        self.store_i64_local_at_offset(
            fulfilled_callback_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            state_local,
            function,
        );

        let rejected_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFromAsyncRejected.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Array.fromAsync rejection callback builtin")
            })?;
        self.emit_function_value_payload(&rejected_meta, function)?;
        function.instruction(&Instruction::LocalSet(rejected_callback_payload_local));
        self.store_i64_local_at_offset(
            rejected_callback_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            state_local,
            function,
        );

        for (offset, local) in [
            (ARRAY_FROM_ASYNC_CAPABILITY_OFFSET, capability_record_local),
            (
                ARRAY_FROM_ASYNC_THROWAWAY_CAPABILITY_OFFSET,
                throwaway_capability_local,
            ),
            (ARRAY_FROM_ASYNC_SOURCE_PAYLOAD_OFFSET, source_payload_local),
            (ARRAY_FROM_ASYNC_SOURCE_TAG_OFFSET, source_tag_local),
            (ARRAY_FROM_ASYNC_TARGET_PAYLOAD_OFFSET, target_payload_local),
            (ARRAY_FROM_ASYNC_TARGET_TAG_OFFSET, target_tag_local),
            (ARRAY_FROM_ASYNC_MAPPER_PAYLOAD_OFFSET, mapper_payload_local),
            (ARRAY_FROM_ASYNC_MAPPER_TAG_OFFSET, mapper_tag_local),
            (
                ARRAY_FROM_ASYNC_THIS_ARG_PAYLOAD_OFFSET,
                this_arg_payload_local,
            ),
            (ARRAY_FROM_ASYNC_THIS_ARG_TAG_OFFSET, this_arg_tag_local),
            (
                ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET,
                fulfilled_callback_payload_local,
            ),
            (
                ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET,
                rejected_callback_payload_local,
            ),
            (
                ARRAY_FROM_ASYNC_ITERATOR_PAYLOAD_OFFSET,
                iterator_payload_local,
            ),
            (ARRAY_FROM_ASYNC_ITERATOR_TAG_OFFSET, iterator_tag_local),
            (ARRAY_FROM_ASYNC_NEXT_PAYLOAD_OFFSET, next_payload_local),
            (ARRAY_FROM_ASYNC_NEXT_TAG_OFFSET, next_tag_local),
            (ARRAY_FROM_ASYNC_MODE_OFFSET, iterator_mode_local),
            (ARRAY_FROM_ASYNC_REALM_ENV_OFFSET, self.current_env_local),
        ] {
            self.store_i64_local_at_offset(state_local, offset, local, function);
        }
        self.store_i64_const_at_offset(state_local, ARRAY_FROM_ASYNC_INDEX_OFFSET, 0, function);
        self.store_i64_const_at_offset(state_local, ARRAY_FROM_ASYNC_LENGTH_OFFSET, 0, function);

        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator next is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(iterator_mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT,
            function,
        );
        self.emit_array_from_async_schedule_await(
            state_local,
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_read_iterator_result_property(
            next_result_payload_local,
            next_result_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_array_from_async_read_iterator_result_property(
            next_result_payload_local,
            next_result_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_current_throw_and_return_promise(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE,
            function,
        );
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_array_from_async_schedule_await(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            zero_local,
            value_tag_local,
            value_payload_local,
            done_tag_local,
            done_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            rejected_callback_payload_local,
            fulfilled_callback_payload_local,
            throwaway_promise_tag_local,
            throwaway_promise_payload_local,
            throwaway_capability_local,
            state_local,
            key_local,
            argv_local,
            argc_local,
            is_constructor_local,
            target_tag_local,
            target_payload_local,
            next_tag_local,
            next_payload_local,
            iterator_tag_local,
            iterator_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_array_from_async_fulfilled(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let stage_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_CAPABILITY_OFFSET,
            capability_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            stage_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_finish_callback(state_local, capability_record_local, function)?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_read_iterator_result_property(
            value_payload_local,
            value_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_finish_callback(state_local, capability_record_local, function)?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::End);
        self.emit_array_from_async_read_iterator_result_property(
            value_payload_local,
            value_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE as i64,
        ));
        function.instruction(&Instruction::LocalSet(stage_local));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MAPPER_PAYLOAD_OFFSET,
            mapper_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MAPPER_TAG_OFFSET,
            mapper_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mapper_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_THIS_ARG_PAYLOAD_OFFSET,
            this_arg_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_THIS_ARG_TAG_OFFSET,
            this_arg_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            mapper_payload_local,
            mapper_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (index_payload_local, index_tag_local),
            ],
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_array_from_async_close_or_reject_callback_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_MAPPED_VALUE,
            function,
        );
        self.emit_array_from_async_schedule_await(
            state_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_from_async_define_current_value(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_close_or_reject_callback_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        self.store_i64_local_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            next_index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MODE_OFFSET,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_LENGTH_OFFSET,
            length_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_finish_callback(state_local, capability_record_local, function)?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE,
            function,
        );
        self.emit_array_from_async_read_array_like_value(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.emit_array_from_async_schedule_await(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator produced too many values",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_begin_close_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_schedule_iterator_step_callback(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(mode_local);
        self.release_temp_local(length_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        self.release_temp_local(capability_record_local);
        self.release_temp_local(stage_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    pub(crate) fn emit_array_from_async_rejected(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let stage_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let close_key_local = self.reserve_temp_local();
        let close_return_payload_local = self.reserve_temp_local();
        let close_return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_CAPABILITY_OFFSET,
            capability_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            stage_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MODE_OFFSET,
            mode_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, self.result_local, self.result_tag_local, function);
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_MAPPED_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_begin_close_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_SYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_ITERATOR_PAYLOAD_OFFSET,
            iterator_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_ITERATOR_TAG_OFFSET,
            iterator_tag_local,
            function,
        );
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local,
                iterator_tag_local,
                key_local: close_key_local,
                return_payload_local: close_return_payload_local,
                return_tag_local: close_return_tag_local,
                result_payload_local: close_result_payload_local,
                result_tag_local: close_result_tag_local,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.emit_array_from_async_return_undefined(function);

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(close_result_tag_local);
        self.release_temp_local(close_result_payload_local);
        self.release_temp_local(close_return_tag_local);
        self.release_temp_local(close_return_payload_local);
        self.release_temp_local(close_key_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(mode_local);
        self.release_temp_local(stage_local);
        self.release_temp_local(capability_record_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    fn emit_array_from_async_schedule_await(
        &mut self,
        state_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let throwaway_capability_local = self.reserve_temp_local();
        let fulfilled_payload_local = self.reserve_temp_local();
        let rejected_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_THROWAWAY_CAPABILITY_OFFSET,
            throwaway_capability_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET,
            fulfilled_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET,
            rejected_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callback_tag_local));
        self.emit_intrinsic_await_with_handlers(
            value_payload_local,
            value_tag_local,
            fulfilled_payload_local,
            callback_tag_local,
            rejected_payload_local,
            callback_tag_local,
            throwaway_capability_local,
            function,
        )?;

        self.release_temp_local(callback_tag_local);
        self.release_temp_local(rejected_payload_local);
        self.release_temp_local(fulfilled_payload_local);
        self.release_temp_local(throwaway_capability_local);
        Ok(())
    }

    fn emit_array_from_async_schedule_iterator_step_callback(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        for (offset, local) in [
            (
                ARRAY_FROM_ASYNC_ITERATOR_PAYLOAD_OFFSET,
                iterator_payload_local,
            ),
            (ARRAY_FROM_ASYNC_ITERATOR_TAG_OFFSET, iterator_tag_local),
            (ARRAY_FROM_ASYNC_NEXT_PAYLOAD_OFFSET, next_payload_local),
            (ARRAY_FROM_ASYNC_NEXT_TAG_OFFSET, next_tag_local),
            (ARRAY_FROM_ASYNC_MODE_OFFSET, mode_local),
        ] {
            self.load_i64_to_local_from_offset(state_local, offset, local, function);
        }
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT,
            function,
        );
        self.emit_array_from_async_schedule_await(
            state_local,
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.fromAsync iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_read_iterator_result_property(
            next_result_payload_local,
            next_result_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.emit_array_from_async_read_iterator_result_property(
            next_result_payload_local,
            next_result_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE,
            function,
        );
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_array_from_async_schedule_await(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_return_undefined(function);

        for local in [
            value_tag_local,
            value_payload_local,
            done_tag_local,
            done_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            mode_local,
            next_tag_local,
            next_payload_local,
            iterator_tag_local,
            iterator_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_array_from_async_close_or_reject_callback_current_throw(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mode_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MODE_OFFSET,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_from_async_begin_close_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(mode_local);
        Ok(())
    }

    fn emit_array_from_async_begin_close_current_throw(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let return_payload_local = self.reserve_temp_local();
        let return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.store_i64_local_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_SAVED_ERROR_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.store_i64_local_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_SAVED_ERROR_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        for (offset, local) in [
            (
                ARRAY_FROM_ASYNC_ITERATOR_PAYLOAD_OFFSET,
                iterator_payload_local,
            ),
            (ARRAY_FROM_ASYNC_ITERATOR_TAG_OFFSET, iterator_tag_local),
            (ARRAY_FROM_ASYNC_MODE_OFFSET, mode_local),
        ] {
            self.load_i64_to_local_from_offset(state_local, offset, local, function);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local,
            return_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_saved_error_on_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_is_callable_i32(return_tag_local, return_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            return_payload_local,
            return_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            close_result_payload_local,
            close_result_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_saved_error_on_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT,
            function,
        );
        self.emit_array_from_async_schedule_await(
            state_local,
            close_result_payload_local,
            close_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(close_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_read_iterator_result_property(
            close_result_payload_local,
            close_result_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_saved_error_on_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        self.emit_array_from_async_read_iterator_result_property(
            close_result_payload_local,
            close_result_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_from_async_reject_saved_error_on_current_throw(
            state_local,
            capability_record_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            state_local,
            ARRAY_FROM_ASYNC_STAGE_OFFSET,
            ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE,
            function,
        );
        self.emit_array_from_async_schedule_await(
            state_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_from_async_return_undefined(function);

        for local in [
            value_tag_local,
            value_payload_local,
            done_tag_local,
            done_payload_local,
            close_result_tag_local,
            close_result_payload_local,
            return_tag_local,
            return_payload_local,
            key_local,
            mode_local,
            iterator_tag_local,
            iterator_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_array_from_async_reject_saved_error_on_current_throw(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_from_async_reject_saved_error(
            state_local,
            capability_record_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_array_from_async_reject_saved_error(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_SAVED_ERROR_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_SAVED_ERROR_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Throw, function);
        self.emit_array_from_async_reject_callback_current_throw(capability_record_local, function)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_from_async_read_iterator_result_property(
        &mut self,
        iterator_result_payload_local: u32,
        iterator_result_tag_local: u32,
        property: &'static str,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload(property)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_result_payload_local,
            iterator_result_tag_local,
            iterator_result_payload_local,
            iterator_result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_array_from_async_read_array_like_value(
        &mut self,
        state_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_SOURCE_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_SOURCE_TAG_OFFSET,
            source_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_number_to_string_payload(index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.release_temp_local(key_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        Ok(())
    }

    fn emit_array_from_async_define_current_value(
        &mut self,
        state_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let boolean_payload_local = self.reserve_temp_local();
        let boolean_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let define_property_payload_local = self.reserve_temp_local();
        let define_property_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_number_to_string_payload(index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(boolean_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(boolean_tag_local));
        for (name, payload, tag) in [
            ("value", value_payload_local, value_tag_local),
            ("writable", boolean_payload_local, boolean_tag_local),
            ("enumerable", boolean_payload_local, boolean_tag_local),
            ("configurable", boolean_payload_local, boolean_tag_local),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_object_define_data(
                descriptor_payload_local,
                self.scratch_local,
                payload,
                tag,
                function,
            )?;
        }
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                )
            })?;
        self.emit_function_value_payload(&define_property_meta, function)?;
        function.instruction(&Instruction::LocalSet(define_property_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_function_handle_call_without_throw_propagation(
            define_property_payload_local,
            define_property_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
                (key_local, key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(define_property_tag_local);
        self.release_temp_local(define_property_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(boolean_tag_local);
        self.release_temp_local(boolean_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_array_from_async_finish_callback(
        &mut self,
        state_local: u32,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_LENGTH_OFFSET,
            length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_MODE_OFFSET,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            state_local,
            ARRAY_FROM_ASYNC_INDEX_OFFSET,
            length_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_array_from_async_set_length(
            target_payload_local,
            target_tag_local,
            length_local,
            function,
        )?;
        self.emit_array_from_async_reject_callback_current_throw(
            capability_record_local,
            function,
        )?;
        self.emit_array_from_async_resolve(
            capability_record_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;

        self.release_temp_local(mode_local);
        self.release_temp_local(length_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_array_from_async_set_length(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(length_tag_local));
        self.emit_object_write(
            target_payload_local,
            target_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;

        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_array_from_async_resolve(
        &mut self,
        capability_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(value_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        Ok(())
    }

    fn emit_array_from_async_reject_current_throw_and_return_promise(
        &mut self,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            reject_payload_local,
            reject_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(error_payload_local, error_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(error_tag_local);
        self.release_temp_local(error_payload_local);
        Ok(())
    }

    fn emit_array_from_async_reject_callback_current_throw(
        &mut self,
        capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            reject_payload_local,
            reject_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(error_payload_local, error_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_array_from_async_return_undefined(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(error_tag_local);
        self.release_temp_local(error_payload_local);
        Ok(())
    }

    fn emit_array_from_async_return_undefined(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }
}
