use super::*;

enum PromiseKeyedElementProjection {
    FulfilledValue,
    SettlementRecord(PromiseSettlement),
}
impl FunctionBuilder<'_> {
    pub(crate) fn emit_promise_all_keyed_resolve_element(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed_element(PromiseKeyedElementProjection::FulfilledValue, function)
    }
    pub(crate) fn emit_promise_all_settled_keyed_element(
        &mut self,
        settlement: PromiseSettlement,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed_element(
            PromiseKeyedElementProjection::SettlementRecord(settlement),
            function,
        )
    }
    fn emit_promise_keyed_element(
        &mut self,
        projection: PromiseKeyedElementProjection,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let element = self.emit_promise_keyed_element_context(function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        schema
            .struct_type::<PromiseKeyedElementContext>()
            .field(PromiseKeyedElementContextSchema::ALREADY_CALLED)
            .read(&element, schema, function);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().initialize(function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<PromiseKeyedElementContext>()
            .field(PromiseKeyedElementContextSchema::ALREADY_CALLED)
            .write(&element, GcOperand::boolean(true), schema, function);
        let shared = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseKeyedElementContext>()
                .field(PromiseKeyedElementContextSchema::SHARED)
                .read(&element, schema, function)
                .reference(),
            function,
        );
        let key_value = schema.reserve_value_local(function);
        let result = schema.reserve_value_local(function);
        let settle = schema.reserve_value_local(function);
        let key_record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseKeyedElementContext>()
                .field(PromiseKeyedElementContextSchema::KEY)
                .read(&element, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&key_record, &key_value, schema, function);
        let key = self.emit_value_to_property_key_locals(&key_value, function)?;
        for (field, out) in [
            (PromiseKeyedCombinatorSharedSchema::VALUES, &result),
            (PromiseKeyedCombinatorSharedSchema::SETTLE, &settle),
        ] {
            let record = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<PromiseKeyedCombinatorShared>()
                    .field(field)
                    .read(&shared, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&record, out, schema, function);
            record.clear(function);
        }
        let value = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &value, function);
        match projection {
            PromiseKeyedElementProjection::FulfilledValue => {}
            PromiseKeyedElementProjection::SettlementRecord(settlement) => {
                let allocation =
                    self.emit_self_backed_promise_settlement_record_allocation_context(function);
                let record = self.emit_alloc_promise_settlement_record(
                    allocation, settlement, &value, function,
                )?;
                value.set_reference(&record, schema, function);
                record.clear(function);
            }
        }
        self.emit_create_data_property_or_throw(&result, &key, &value, &pending, function)?;
        self.emit_promise_abrupt_exit(&pending, exit, function);
        let remaining = schema.reserve_i64_local(function);
        schema
            .struct_type::<PromiseKeyedCombinatorShared>()
            .field(PromiseKeyedCombinatorSharedSchema::REMAINING)
            .read(&shared, schema, function)
            .store_i64(remaining, function);
        remaining.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        remaining.store(function);
        schema
            .struct_type::<PromiseKeyedCombinatorShared>()
            .field(PromiseKeyedCombinatorSharedSchema::REMAINING)
            .write(&shared, GcOperand::i64_local(remaining), schema, function);
        remaining.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        let args = self.emit_pre_evaluated_arg_vector(&[&result], function);
        self.emit_function_or_proxy_call_with_argv(&settle, &undefined, &args, &pending, function)?;
        args.clear(function);
        undefined.clear(function);
        self.emit_promise_abrupt_exit(&pending, exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().initialize(function);
        schema.release_i64_local(remaining, function);
        value.clear(function);
        settle.clear(function);
        result.clear(function);
        key.clear(function);
        key_record.clear(function);
        key_value.clear(function);
        shared.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.clear(function);
        element.clear(function);
        Ok(())
    }
}
