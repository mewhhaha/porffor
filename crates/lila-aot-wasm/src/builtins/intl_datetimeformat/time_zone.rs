use super::*;
use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, LOOKUP_TIME_ZONE_HEADER_BYTES,
    LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET,
    MAX_TIME_ZONE_IDENTIFIER_BYTES,
};

/// What a `LookupNamedTimeZone` rejection means to the caller. Both
/// `Intl.DateTimeFormat` and Temporal resolve named identifiers through this
/// one call; they differ only in whether an unknown name is final.
#[derive(Clone, Copy)]
pub(in crate::builtins) enum NamedTimeZoneRejection {
    /// A RangeError with this message.
    Throw(&'static str),
    /// Set this local to 1 when the name resolved and 0 when it did not,
    /// leaving the identifier untouched in the latter case.
    Report(u32),
}

impl FunctionBuilder<'_> {
    /// `GetAvailableNamedTimeZoneIdentifier`: replaces `identifier_payload`
    /// with the record's normalized `[[Identifier]]` and, when asked, stores its
    /// `[[PrimaryIdentifier]]`.
    pub(in crate::builtins) fn emit_intl_lookup_named_time_zone(
        &mut self,
        identifier_payload: u32,
        primary_payload: Option<u32>,
        rejection: NamedTimeZoneRejection,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let identifier_length = self.reserve_temp_local();
        let primary_length = self.reserve_temp_local();
        let import = self.intl_call_import_function_index()?;
        function.instruction(&Instruction::I64Const(
            IntlHostOp::LookupNamedTimeZone.wire(),
        ));
        function.instruction(&Instruction::LocalGet(identifier_payload));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        match rejection {
            NamedTimeZoneRejection::Throw(message) => {
                self.emit_throw_current_function_realm_range_error(
                    message,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
            NamedTimeZoneRejection::Report(found) => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(found));
            }
        }
        function.instruction(&Instruction::Else);
        // Query and retry are pure. A rejection after a capacity answer or an
        // incompatible length is a provider fault, never a second JS coercion.
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
        function.instruction(&Instruction::LocalSet(length));
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalSet(response));
        function.instruction(&Instruction::I64Const(
            IntlHostOp::LookupNamedTimeZone.wire(),
        ));
        function.instruction(&Instruction::LocalGet(identifier_payload));
        self.emit_pack_string_payload(response, length, function);
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
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
        if let Some(primary_payload) = primary_payload {
            function.instruction(&Instruction::LocalGet(response));
            function.instruction(&Instruction::LocalGet(identifier_length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(response));
            self.emit_pack_string_payload(response, primary_length, function);
            function.instruction(&Instruction::LocalSet(primary_payload));
        }
        if let NamedTimeZoneRejection::Report(found) = rejection {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found));
        }
        function.instruction(&Instruction::End);
        for local in [primary_length, identifier_length, length, response, outcome] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_intl_dtf_lookup_named_time_zone(
        &mut self,
        identifier_payload: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_lookup_named_time_zone(
            identifier_payload,
            None,
            NamedTimeZoneRejection::Throw(INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE),
            function,
        )
    }
}
