use super::*;

#[must_use = "Atomics.waitAsync result Object prototype must be consumed"]
struct AtomicsWaitAsyncResultObjectPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    fn emit_atomics_wait_async_result_object_prototype(
        &mut self,
        function: &mut Function,
    ) -> AtomicsWaitAsyncResultObjectPrototypeLocal {
        let object_prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        AtomicsWaitAsyncResultObjectPrototypeLocal(object_prototype_local)
    }

    pub(super) fn emit_atomics_wait_async_return_object(
        &mut self,
        outcome: AtomicsWaitOutcome,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result_object_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let AtomicsWaitAsyncResultObjectPrototypeLocal(object_prototype_local) =
            self.emit_atomics_wait_async_result_object_prototype(function);

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(result_object_local));
        self.release_temp_local(object_prototype_local);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "async",
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(outcome.spelling()),
        ));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "value",
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(result_object_local);
        Ok(())
    }

    pub(super) fn emit_atomics_wait_async_return_promise(
        &mut self,
        address_local: u32,
        deadline_nanos_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result_object_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let waiter_local = self.reserve_temp_local();
        let waiter_tail_local = self.reserve_temp_local();
        let waiter_next_local = self.reserve_temp_local();
        let waiter_host_id_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let async_payload_local = self.reserve_temp_local();
        let AtomicsWaitAsyncResultObjectPrototypeLocal(object_prototype_local) =
            self.emit_atomics_wait_async_result_object_prototype(function);
        let promise_allocation_context =
            self.emit_current_function_realm_intrinsic_promise_allocation_context(function);

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(result_object_local));
        self.emit_alloc_promise_with_prototype(
            promise_allocation_context,
            promise_payload_local,
            promise_record_local,
            function,
        )?;
        self.release_temp_local(object_prototype_local);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(async_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "async",
            async_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;

        self.emit_heap_alloc_const(HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(waiter_local));
        self.store_i64_const_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
            1,
            function,
        );
        self.store_i64_local_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET,
            address_local,
            function,
        );
        self.store_i64_local_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET,
            deadline_nanos_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(waiter_host_id_local));
        if let Some(agent_call_function_index) = self.functions.agent_call_import_function_index() {
            function.instruction(&Instruction::I64Const(
                AgentHostOperation::RegisterAsyncWaiter.wire(),
            ));
            function.instruction(&Instruction::LocalGet(address_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Call(agent_call_function_index));
            function.instruction(&Instruction::LocalSet(waiter_host_id_local));
        }
        self.store_i64_local_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
            waiter_host_id_local,
            function,
        );
        self.store_i64_const_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(waiter_tail_local));
        function.instruction(&Instruction::LocalGet(waiter_tail_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::GlobalSet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            waiter_tail_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_next_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::LocalSet(waiter_tail_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            waiter_tail_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "value",
            promise_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(async_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(waiter_host_id_local);
        self.release_temp_local(waiter_next_local);
        self.release_temp_local(waiter_tail_local);
        self.release_temp_local(waiter_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(result_object_local);
        Ok(())
    }
}
