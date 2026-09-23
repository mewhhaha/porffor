use super::*;
use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, LOOKUP_TIME_ZONE_HEADER_BYTES,
    LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET,
    MAX_TIME_ZONE_IDENTIFIER_BYTES,
};

impl FunctionBuilder<'_> {
    fn emit_intl_dtf_time_zone_call(
        &mut self,
        request_payload: u32,
        response_pointer: u32,
        response_length: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let import = self.intl_call_import_function_index()?;
        function.instruction(&Instruction::I64Const(
            IntlHostOp::LookupNamedTimeZone.wire(),
        ));
        function.instruction(&Instruction::LocalGet(request_payload));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // Query and retry are pure. A rejection after constructor validation or
        // an incompatible length is a provider fault, never a second JS coercion.
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(LOOKUP_TIME_ZONE_HEADER_BYTES as u32 + 2).wire(),
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(
                (LOOKUP_TIME_ZONE_HEADER_BYTES + 2 * MAX_TIME_ZONE_IDENTIFIER_BYTES) as u32,
            )
            .wire(),
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(0).wire(),
        ));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(response_length));
        self.emit_heap_alloc_from_local(response_length, function)?;
        function.instruction(&Instruction::LocalSet(response_pointer));
        function.instruction(&Instruction::I64Const(
            IntlHostOp::LookupNamedTimeZone.wire(),
        ));
        function.instruction(&Instruction::LocalGet(request_payload));
        self.emit_pack_string_payload(response_pointer, response_length, function);
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalGet(response_length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(outcome);
        Ok(())
    }

    pub(super) fn emit_intl_dtf_lookup_named_time_zone(
        &mut self,
        identifier_payload: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let response = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let identifier_length = self.reserve_temp_local();
        let primary_length = self.reserve_temp_local();
        self.emit_intl_dtf_time_zone_call(identifier_payload, response, length, function)?;
        self.load_i64_to_local_from_offset(
            response,
            LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET,
            identifier_length,
            function,
        );
        self.load_i64_to_local_from_offset(
            response,
            LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET,
            primary_length,
            function,
        );
        for field in [identifier_length, primary_length] {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Const(
                MAX_TIME_ZONE_IDENTIFIER_BYTES as i64,
            ));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(identifier_length));
        function.instruction(&Instruction::LocalGet(primary_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(LOOKUP_TIME_ZONE_HEADER_BYTES as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(response));
        function.instruction(&Instruction::I64Const(LOOKUP_TIME_ZONE_HEADER_BYTES as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(response));
        self.emit_pack_string_payload(response, identifier_length, function);
        function.instruction(&Instruction::LocalSet(identifier_payload));
        for local in [primary_length, identifier_length, length, response] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
