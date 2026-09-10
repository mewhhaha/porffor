use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_object_has_property_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_tag_local = self.reserve_temp_local();
        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        let emit_result = self.emit_object_has_property_with_key_tag_i32(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        );
        self.release_temp_local(key_tag_local);
        emit_result
    }

    pub(crate) fn emit_object_has_property_with_key_tag_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = RuntimeHelperId::ObjectHasProperty.index(
            self.heap_alloc_function_index
                .expect("HasProperty requires the object runtime"),
        );
        let helper_payload_local = self.reserve_temp_local();
        let helper_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        self.emit_outlined_object_read_realm_argument(function);
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(helper_payload_local, helper_tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed(
            helper_payload_local,
            helper_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(helper_payload_local));
        function.instruction(&Instruction::LocalSet(result_local));
        self.release_temp_local(helper_tag_local);
        self.release_temp_local(helper_payload_local);
        Ok(())
    }

    /// Emits the only body of the full HasProperty dispatch. The helper owns
    /// traversal scratch locals; callers keep their selected Reference live.
    pub(crate) fn compile_object_has_property_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectHasProperty);
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_has_property_dispatch_with_key_tag_i32(
            0,
            1,
            2,
            3,
            self.result_local,
            &mut function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_has_property_dispatch_with_key_tag_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let internal_key_local = self.reserve_temp_local();
        let trap_key_payload_local = self.reserve_temp_local();
        let dispatch_state_local = self.reserve_temp_local();
        let named_payload_local = self.reserve_temp_local();
        let named_tag_local = self.reserve_temp_local();
        let integer_indexed_has_handled_local = self.reserve_temp_local();

        self.emit_property_key_value_payload_to_local(key_local, trap_key_payload_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dispatch_state_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(dispatch_state_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dispatch_state_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_object_boxed_kind_for_tag(
            current_local,
            current_tag_local,
            boxed_kind_local,
            function,
        );
        for branch in ObjectInternalMethodBranch::ORDER.iter().copied() {
            match branch {
                ObjectInternalMethodBranch::Proxy => {
                    function.instruction(&Instruction::LocalGet(boxed_kind_local));
                    function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
                    function.instruction(&Instruction::I64GeU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_load_live_proxy_slots(
                        current_local,
                        ProxySlotLocals::new(
                            ProxyTargetLocals::new(target_payload_local, target_tag_local),
                            ProxyHandlerLocals::new(boxed_kind_local, handler_tag_local),
                        ),
                        ProxyRevocationRoute::CurrentFunctionRealm,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(self.strings.payload("has")));
                    function.instruction(&Instruction::LocalSet(internal_key_local));
                    self.emit_object_read_without_throw_propagation(
                        boxed_kind_local,
                        handler_tag_local,
                        boxed_kind_local,
                        handler_tag_local,
                        internal_key_local,
                        trap_result_payload_local,
                        trap_result_tag_local,
                        function,
                    )?;
                    // A getter reached by GetMethod may throw. Leave the raw
                    // Proxy/loop/block traversal before deciding whether its
                    // result is absent or callable; the shared propagation
                    // below consumes these same result locals.
                    self.emit_break_current_completion_if_throw(3, function);
                    function.instruction(&Instruction::LocalGet(trap_result_payload_local));
                    function.instruction(&Instruction::LocalSet(trap_payload_local));
                    function.instruction(&Instruction::LocalGet(trap_result_tag_local));
                    function.instruction(&Instruction::LocalSet(trap_tag_local));

                    self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_function_or_proxy_call_leave_throw_completion(
                        trap_payload_local,
                        trap_tag_local,
                        boxed_kind_local,
                        handler_tag_local,
                        &[
                            (target_payload_local, target_tag_local),
                            (trap_key_payload_local, key_tag_local),
                        ],
                        trap_result_payload_local,
                        trap_result_tag_local,
                        function,
                    )?;
                    // Leave the traversal before propagating: this check is
                    // inside the callable branch, Proxy branch, loop and block.
                    self.emit_break_current_completion_if_throw(4, function);
                    self.compile_truthy_tagged_i32(
                        trap_result_tag_local,
                        trap_result_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(result_local));
                    self.emit_proxy_has_invariant_check(
                        target_payload_local,
                        target_tag_local,
                        key_local,
                        result_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(trap_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(trap_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(target_payload_local));
                    function.instruction(&Instruction::LocalSet(current_local));
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::LocalSet(current_tag_local));
                    function.instruction(&Instruction::I64Const(2));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::Else);
                    self.emit_proxy_execution_realm_type_error(
                        "Proxy has trap is not callable",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                ObjectInternalMethodBranch::IntegerIndexed => {
                    self.emit_typed_array_integer_index_validity_i32(
                        current_local,
                        current_tag_local,
                        key_local,
                        key_tag_local,
                        result_local,
                        integer_indexed_has_handled_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(integer_indexed_has_handled_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                }
                ObjectInternalMethodBranch::Array => {
                    function.instruction(&Instruction::LocalGet(current_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_property_key_payload_equality_i32(
                        self.scratch_local,
                        key_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::Else);
                    self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_has_index_i32(
                        current_local,
                        index_local,
                        result_local,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_named_prop_read(
                        current_local,
                        key_local,
                        named_payload_local,
                        named_tag_local,
                        Some(result_local),
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::Else);
                    self.emit_load_prototype_to_current_locals(
                        current_local,
                        current_tag_local,
                        prototype_local,
                        function,
                    );
                    function.instruction(&Instruction::I64Const(2));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                ObjectInternalMethodBranch::Arguments => {
                    function.instruction(&Instruction::LocalGet(current_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));

                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_property_key_payload_equality_i32(
                        self.scratch_local,
                        key_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
                        result_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_property_key_payload_equality_i32(
                        self.scratch_local,
                        key_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
                        result_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_has_index_i32(
                        current_local,
                        index_local,
                        result_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_named_prop_read(
                        current_local,
                        key_local,
                        named_payload_local,
                        named_tag_local,
                        Some(result_local),
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::Else);
                    self.emit_load_prototype_to_current_locals(
                        current_local,
                        current_tag_local,
                        prototype_local,
                        function,
                    );
                    function.instruction(&Instruction::I64Const(2));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                ObjectInternalMethodBranch::BoxedString => {
                    function.instruction(&Instruction::LocalGet(boxed_kind_local));
                    function
                        .instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                        target_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_property_key_payload_equality_i32(
                        self.scratch_local,
                        key_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::Else);
                    self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_unpack_string_payload(
                        target_payload_local,
                        buffer_local,
                        len_local,
                        function,
                    );
                    self.emit_utf16_code_unit_len_from_utf8_locals(
                        buffer_local,
                        len_local,
                        entry_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::LocalGet(entry_local));
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                ObjectInternalMethodBranch::Ordinary => {
                    // Function's own `prototype` slot is part of ordinary
                    // storage and must be tested at every prototype step.
                    function.instruction(&Instruction::LocalGet(current_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_property_key_payload_equality_i32(
                        self.scratch_local,
                        key_local,
                        function,
                    );
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                        boxed_kind_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(boxed_kind_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(dispatch_state_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_PTR_OFFSET,
                        buffer_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        current_local,
                        HEAP_LEN_OFFSET,
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
                    self.emit_property_key_payload_equality_i32(
                        key_payload_local,
                        key_local,
                        function,
                    );
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

                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::Else);
                    self.emit_load_prototype_to_current_locals(
                        current_local,
                        current_tag_local,
                        prototype_local,
                        function,
                    );
                    function.instruction(&Instruction::I64Const(2));
                    function.instruction(&Instruction::LocalSet(dispatch_state_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
            }

            function.instruction(&Instruction::LocalGet(dispatch_state_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        // The closed order always ends in Ordinary, which either finishes or
        // advances the current object. Keep the loop total if that changes.
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;

        self.release_temp_local(integer_indexed_has_handled_local);
        self.release_temp_local(named_tag_local);
        self.release_temp_local(named_payload_local);
        self.release_temp_local(dispatch_state_local);
        self.release_temp_local(trap_key_payload_local);
        self.release_temp_local(internal_key_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_local);
        Ok(())
    }
}
