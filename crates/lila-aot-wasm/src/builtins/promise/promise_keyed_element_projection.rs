use super::*;

enum PromiseKeyedElementProjection {
    FulfilledValue,
    SettlementRecord(PromiseSettlement),
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_promise_all_keyed_resolve_element(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_all_keyed_element(PromiseKeyedElementProjection::FulfilledValue, function)
    }

    pub(crate) fn emit_promise_all_settled_keyed_element(
        &mut self,
        settlement: PromiseSettlement,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_all_keyed_element(
            PromiseKeyedElementProjection::SettlementRecord(settlement),
            function,
        )
    }

    fn emit_promise_all_keyed_element(
        &mut self,
        projection: PromiseKeyedElementProjection,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_context_local = self.reserve_temp_local();
        let already_called_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        self.emit_load_promise_internal_function_context(element_context_local, function);
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            already_called_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_called_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            result_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);

        match projection {
            PromiseKeyedElementProjection::FulfilledValue => {}
            PromiseKeyedElementProjection::SettlementRecord(settlement) => {
                let record_payload_local = self.reserve_temp_local();
                let status_payload_local = self.reserve_temp_local();
                let status_tag_local = self.reserve_temp_local();
                let (status, result_property) = match settlement {
                    PromiseSettlement::Fulfill => ("fulfilled", "value"),
                    PromiseSettlement::Reject => ("rejected", "reason"),
                };
                let allocation_context =
                    self.emit_self_backed_promise_settlement_record_allocation_context(function);
                self.emit_alloc_promise_settlement_record(allocation_context, function)?;
                function.instruction(&Instruction::LocalSet(record_payload_local));
                function.instruction(&Instruction::I64Const(self.strings.payload(status)));
                function.instruction(&Instruction::LocalSet(status_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(status_tag_local));
                self.emit_object_define_local_data_with_flags(
                    record_payload_local,
                    "status",
                    status_payload_local,
                    status_tag_local,
                    true,
                    true,
                    true,
                    function,
                )?;
                self.emit_object_define_local_data_with_flags(
                    record_payload_local,
                    result_property,
                    value_payload_local,
                    value_tag_local,
                    true,
                    true,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(record_payload_local));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.release_temp_local(status_tag_local);
                self.release_temp_local(status_payload_local);
                self.release_temp_local(record_payload_local);
            }
        }
        self.emit_object_define_enumerable_data(
            result_payload_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(result_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            value_tag_local,
            value_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            remaining_local,
            result_payload_local,
            shared_context_local,
            key_tag_local,
            key_payload_local,
            already_called_local,
            element_context_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
