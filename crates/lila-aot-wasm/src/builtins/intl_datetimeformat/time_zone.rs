use super::*;
use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, LOOKUP_TIME_ZONE_HEADER_BYTES,
    LOOKUP_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, LOOKUP_TIME_ZONE_PRIMARY_LENGTH_OFFSET,
    MAX_TIME_ZONE_IDENTIFIER_BYTES, RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET,
    RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET, RESOLVE_TIME_ZONE_HEADER_BYTES,
    RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET, RESOLVE_TIME_ZONE_KIND_OFFSET,
    RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET, RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET,
    RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES,
};

enum DtfTimeZoneProviderCall {
    Lookup,
    Snapshot,
}

impl DtfTimeZoneProviderCall {
    const fn operation(&self) -> IntlHostOp {
        match self {
            Self::Lookup => IntlHostOp::LookupNamedTimeZone,
            Self::Snapshot => IntlHostOp::ResolveTimeZone,
        }
    }
    const fn minimum_response_bytes(&self) -> u32 {
        match self {
            Self::Lookup => LOOKUP_TIME_ZONE_HEADER_BYTES as u32 + 2,
            Self::Snapshot => RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES as u32,
        }
    }
    const fn maximum_response_bytes(&self) -> u32 {
        match self {
            Self::Lookup => {
                (LOOKUP_TIME_ZONE_HEADER_BYTES + 2 * MAX_TIME_ZONE_IDENTIFIER_BYTES) as u32
            }
            Self::Snapshot => u32::MAX,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_components_in_time_zone(
        &mut self,
        record: u32,
        time: u32,
        exact_time: u32,
        name_style: u32,
        components: &DtfComponentLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_seconds = self.reserve_temp_local();
        self.emit_dtf_set_const(offset_seconds, 0, function);
        self.emit_dtf_set_string(components.time_zone_name, "", function);
        self.emit_dtf_if_nonzero(exact_time, function);
        self.emit_intl_dtf_time_zone_snapshot(
            record,
            time,
            name_style,
            offset_seconds,
            components.time_zone_name,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_dtf_components_from_time(time, offset_seconds, components, function);
        self.release_temp_local(offset_seconds);
        Ok(())
    }

    fn emit_intl_dtf_time_zone_call(
        &mut self,
        call: DtfTimeZoneProviderCall,
        request_payload: u32,
        response_pointer: u32,
        response_length: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let import = self.intl_call_import_function_index()?;
        function.instruction(&Instruction::I64Const(call.operation().wire()));
        function.instruction(&Instruction::LocalGet(request_payload));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalSet(outcome));
        match call {
            DtfTimeZoneProviderCall::Lookup => {
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
            }
            DtfTimeZoneProviderCall::Snapshot => {}
        }
        // Query and retry are pure. A rejection after constructor validation or
        // an incompatible length is a provider fault, never a second JS coercion.
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(call.minimum_response_bytes()).wire(),
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(call.maximum_response_bytes()).wire(),
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
        function.instruction(&Instruction::I64Const(call.operation().wire()));
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
        self.emit_intl_dtf_time_zone_call(
            DtfTimeZoneProviderCall::Lookup,
            identifier_payload,
            response,
            length,
            function,
        )?;
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

    pub(super) fn emit_intl_dtf_time_zone_snapshot(
        &mut self,
        record: u32,
        time: u32,
        style: u32,
        offset_seconds: u32,
        display_name: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = self.reserve_temp_local();
        let fixed = self.reserve_temp_local();
        let epoch = self.reserve_temp_local();
        let identifier = self.reserve_temp_local();
        let identifier_pointer = self.reserve_temp_local();
        let identifier_length = self.reserve_temp_local();
        let locale = self.reserve_temp_local();
        let locale_pointer = self.reserve_temp_local();
        let locale_length = self.reserve_temp_local();
        let request_pointer = self.reserve_temp_local();
        let request_length = self.reserve_temp_local();
        let request_payload = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        let response_pointer = self.reserve_temp_local();
        let response_length = self.reserve_temp_local();
        for (offset, local) in [
            (HEAP_INTL_DTF_TIME_ZONE_KIND_OFFSET, kind),
            (HEAP_INTL_DTF_TIME_ZONE_FIXED_SECONDS_OFFSET, fixed),
            (HEAP_INTL_DTF_TIME_ZONE_OFFSET, identifier),
            (HEAP_INTL_DTF_LOCALE_OFFSET, locale),
        ] {
            self.load_i64_to_local_from_offset(record, offset, local, function);
        }
        self.emit_unpack_string_payload(
            identifier,
            identifier_pointer,
            identifier_length,
            function,
        );
        self.emit_dtf_if_code_eq(kind, TimeZoneKind::FixedOffset.code(), function);
        self.emit_dtf_set_const(identifier_length, 0, function);
        function.instruction(&Instruction::End);
        self.emit_unpack_string_payload(locale, locale_pointer, locale_length, function);
        function.instruction(&Instruction::LocalGet(time));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(epoch));
        function.instruction(&Instruction::I64Const(
            RESOLVE_TIME_ZONE_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::LocalGet(identifier_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(locale_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(request_length));
        self.emit_heap_alloc_from_local(request_length, function)?;
        function.instruction(&Instruction::LocalSet(request_pointer));
        for (offset, local) in [
            (RESOLVE_TIME_ZONE_KIND_OFFSET, kind),
            (RESOLVE_TIME_ZONE_FIXED_SECONDS_OFFSET, fixed),
            (RESOLVE_TIME_ZONE_EPOCH_SECONDS_OFFSET, epoch),
            (RESOLVE_TIME_ZONE_NAME_STYLE_OFFSET, style),
            (
                RESOLVE_TIME_ZONE_IDENTIFIER_LENGTH_OFFSET,
                identifier_length,
            ),
            (RESOLVE_TIME_ZONE_LOCALE_LENGTH_OFFSET, locale_length),
        ] {
            self.store_i64_local_at_offset(request_pointer, offset, local, function);
        }
        function.instruction(&Instruction::LocalGet(request_pointer));
        function.instruction(&Instruction::I64Const(
            RESOLVE_TIME_ZONE_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_copy_bytes(identifier_pointer, cursor, identifier_length, function);
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(identifier_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_copy_bytes(locale_pointer, cursor, locale_length, function);
        self.emit_pack_string_payload(request_pointer, request_length, function);
        function.instruction(&Instruction::LocalSet(request_payload));
        self.emit_intl_dtf_time_zone_call(
            DtfTimeZoneProviderCall::Snapshot,
            request_payload,
            response_pointer,
            response_length,
            function,
        )?;
        self.load_i64_to_local_from_offset(response_pointer, 0, offset_seconds, function);
        function.instruction(&Instruction::LocalGet(offset_seconds));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalGet(offset_seconds));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(response_length));
        function.instruction(&Instruction::I64Const(
            RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(response_length));
        function.instruction(&Instruction::LocalGet(response_length));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(style));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(response_pointer));
        function.instruction(&Instruction::I64Const(
            RESOLVE_TIME_ZONE_RESULT_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(response_pointer));
        self.emit_pack_string_payload(response_pointer, response_length, function);
        function.instruction(&Instruction::LocalSet(display_name));
        for local in [
            response_length,
            response_pointer,
            cursor,
            request_payload,
            request_length,
            request_pointer,
            locale_length,
            locale_pointer,
            locale,
            identifier_length,
            identifier_pointer,
            identifier,
            epoch,
            fixed,
            kind,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
