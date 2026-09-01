use super::*;

#[must_use = "the prepared AggregateError must be finalized with its errors list"]
pub(super) struct PreparedAggregateErrorLocal {
    object: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_prepare_aggregate_error_instance(
        &mut self,
        prototype_payload_local: u32,
        message_arg_payload_local: u32,
        message_arg_tag_local: u32,
        function: &mut Function,
    ) -> Result<PreparedAggregateErrorLocal, EmitError> {
        let object_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );

        function.instruction(&Instruction::LocalGet(message_arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(
            message_arg_payload_local,
            message_arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(message_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            message_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_install_error_cause_from_arg(
            object_local,
            ErrorCauseOptionsArgument::AggregateError,
            function,
        )?;

        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(message_payload_local);
        Ok(PreparedAggregateErrorLocal {
            object: object_local,
        })
    }

    pub(super) fn emit_prepare_promise_any_aggregate_error_instance(
        &mut self,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<PreparedAggregateErrorLocal, EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        Ok(PreparedAggregateErrorLocal {
            object: object_local,
        })
    }

    pub(super) fn emit_finish_aggregate_error_instance(
        &mut self,
        prepared: PreparedAggregateErrorLocal,
        errors_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let PreparedAggregateErrorLocal {
            object: object_local,
        } = prepared;
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("errors")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
