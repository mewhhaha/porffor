//! Canonical GC zone identities from the pinned provider, with pure UTC/offset paths.
use super::*;
use crate::builtins::temporal::TemporalTimeZoneStringGoal;
use lila_intl::{LOOKUP_TIME_ZONE_HEADER_BYTES, MAX_TIME_ZONE_IDENTIFIER_BYTES};

pub(crate) struct PreparedTemporalZoneLocals {
    identifier: ValueLocals,
    primary: ValueLocals,
    kind: I64Local,
    fixed_seconds: I64Local,
}
impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn reserve_temporal_zone_identity(
        &mut self,
        f: &mut Function,
    ) -> PreparedTemporalZoneLocals {
        let schema = self.runtime_schema();
        let result = PreparedTemporalZoneLocals {
            identifier: schema.reserve_value_local(f),
            primary: schema.reserve_value_local(f),
            kind: schema.reserve_i64_local(f),
            fixed_seconds: schema.reserve_i64_local(f),
        };
        result.identifier.set_undefined(f);
        result.primary.set_undefined(f);
        result
    }
    pub(in crate::builtins) fn emit_temporal_time_zone_or_system_into(
        &mut self,
        prepared: PreparedTemporalZoneLocals,
        input: &ValueLocals,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let system = self.emit_system_time_zone(f)?;
        prepared
            .identifier
            .set_reference(system.identifier(), self.runtime_schema(), f);
        prepared
            .primary
            .set_reference(system.identifier(), self.runtime_schema(), f);
        system.kind_local().load(f);
        prepared.kind.store(f);
        system.fixed_offset_seconds().load(f);
        prepared.fixed_seconds.store(f);
        system.release(self, f);
        f.instruction(&Instruction::Else);
        let explicit = self.emit_temporal_zoned_date_time_time_zone(
            input,
            TemporalTimeZoneStringGoal::Object,
            f,
        )?;
        prepared
            .identifier
            .set_reference(explicit.identifier(), self.runtime_schema(), f);
        prepared
            .primary
            .set_reference(explicit.primary_identifier(), self.runtime_schema(), f);
        explicit.kind_local().load(f);
        prepared.kind.store(f);
        explicit.fixed_offset_seconds().load(f);
        prepared.fixed_seconds.store(f);
        explicit.release(self, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_publish_prepared_zone(prepared, f)
    }
    pub(in crate::builtins) fn emit_temporal_zoned_date_time_time_zone(
        &mut self,
        input: &ValueLocals,
        goal: TemporalTimeZoneStringGoal,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        let prepared = self.reserve_temporal_zone_identity(f);
        self.emit_temporal_zoned_date_time_time_zone_into(prepared, input, goal, f)
    }
    pub(in crate::builtins) fn emit_temporal_zoned_date_time_time_zone_into(
        &mut self,
        prepared: PreparedTemporalZoneLocals,
        input: &ValueLocals,
        goal: TemporalTimeZoneStringGoal,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        prepared.identifier.copy_from(input, f);
        self.emit_temporal_parse_time_zone_value_into(&prepared.identifier, goal, f)?;
        self.emit_temporal_finish_zone_identity(prepared, f)
    }
    pub(in crate::builtins) fn emit_temporal_resolve_zone_identifier(
        &mut self,
        identifier: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        let prepared = self.reserve_temporal_zone_identity(f);
        prepared
            .identifier
            .set_reference(identifier, self.runtime_schema(), f);
        self.emit_temporal_finish_zone_identity(prepared, f)
    }
    pub(in crate::builtins) fn emit_temporal_zone_from_zoned_record(
        &mut self,
        record: &BrandedTemporalZonedRecordLocals,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        let schema = self.runtime_schema();
        let identifier = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalZonedDateTimeObject>()
                .field(TemporalZonedDateTimeObjectSchema::TIME_ZONE)
                .read(record.record(), schema, f)
                .reference(),
            f,
        );
        let result = self.emit_temporal_resolve_zone_identifier(&identifier, f)?;
        identifier.clear(f);
        Ok(result)
    }
    fn emit_temporal_publish_prepared_zone(
        &mut self,
        prepared: PreparedTemporalZoneLocals,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        let schema = self.runtime_schema();
        let identifier = schema.reserve_gc_local(f).initialize(
            prepared.identifier.cast_reference::<StringValue>(schema, f),
            f,
        );
        let primary = schema
            .reserve_gc_local(f)
            .initialize(prepared.primary.cast_reference::<StringValue>(schema, f), f);
        prepared.primary.clear(f);
        prepared.identifier.clear(f);
        Ok(ResolvedTemporalZoneLocals {
            identifier,
            primary,
            kind: prepared.kind,
            fixed_seconds: prepared.fixed_seconds,
        })
    }
    fn emit_temporal_finish_zone_identity(
        &mut self,
        prepared: PreparedTemporalZoneLocals,
        f: &mut Function,
    ) -> Result<ResolvedTemporalZoneLocals, EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_gc_local(f).initialize(
            prepared.identifier.cast_reference::<StringValue>(schema, f),
            f,
        );
        let utc = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("UTC", f)?, f);
        let fold = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(1));
        fold.store(f);
        f.instruction(&Instruction::I64Const(0));
        prepared.fixed_seconds.store(f);
        self.emit_string_payload_equality_i32_with_ascii_case_folding(&input, &utc, Some(fold), f);
        self.open_frame(ControlFrameKind::If, f);
        prepared.identifier.set_reference(&utc, schema, f);
        prepared.primary.set_reference(&utc, schema, f);
        f.instruction(&Instruction::I64Const(TemporalZoneKind::Utc.code()));
        prepared.kind.store(f);
        f.instruction(&Instruction::Else);
        let first = schema.reserve_i64_local(f);
        let zero = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        self.emit_temporal_load_code_unit(&input, zero, first, f);
        first.load(f);
        f.instruction(&Instruction::I64Const(b'+' as i64));
        f.instruction(&Instruction::I64Eq);
        first.load(f);
        f.instruction(&Instruction::I64Const(b'-' as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_fixed_time_zone_offset_seconds(&input, prepared.fixed_seconds, f)?;
        self.emit_temporal_format_fixed_time_zone_offset(
            prepared.fixed_seconds,
            &prepared.identifier,
            f,
        )?;
        prepared.primary.copy_from(&prepared.identifier, f);
        f.instruction(&Instruction::I64Const(TemporalZoneKind::FixedOffset.code()));
        prepared.kind.store(f);
        f.instruction(&Instruction::Else);
        self.emit_temporal_lookup_zone_identity(&prepared, &input, &utc, fold, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i64_local(zero, f);
        schema.release_i64_local(first, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(fold, f);
        utc.clear(f);
        input.clear(f);
        self.emit_temporal_publish_prepared_zone(prepared, f)
    }
    fn emit_temporal_lookup_zone_identity(
        &mut self,
        prepared: &PreparedTemporalZoneLocals,
        input: &GcLocal<StringValue>,
        utc: &GcLocal<StringValue>,
        fold: I32Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let syntax = schema.reserve_i64_local(f);
        self.emit_temporal_named_identifier_syntax(input, syntax, f);
        syntax.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_TIME_ZONE_IDENTIFIER,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let reply = self.emit_temporal_lookup_data(input, f)?;
        let identifier_length = schema.reserve_i64_local(f);
        let primary_length = schema.reserve_i64_local(f);
        let start = schema.reserve_i64_local(f);
        reply.read_constant_word(0, identifier_length, self, f);
        reply.read_constant_word(8, primary_length, self, f);
        identifier_length.load(f);
        primary_length.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(LOOKUP_TIME_ZONE_HEADER_BYTES as i64));
        f.instruction(&Instruction::I64Add);
        reply.length().load(f);
        f.instruction(&Instruction::I64Ne);
        self.emit_temporal_provider_corruption_if_i32(f);
        f.instruction(&Instruction::I64Const(LOOKUP_TIME_ZONE_HEADER_BYTES as i64));
        start.store(f);
        let identifier = reply.read_identifier(start, identifier_length, self, f);
        start.load(f);
        identifier_length.load(f);
        f.instruction(&Instruction::I64Add);
        start.store(f);
        let primary = reply.read_identifier(start, primary_length, self, f);
        for text in [&identifier, &primary] {
            self.emit_pinned_named_identifier_syntax(text, syntax, f);
            syntax.load(f);
            f.instruction(&Instruction::I64Eqz);
            self.emit_temporal_provider_corruption_if_i32(f);
        }
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            input,
            &identifier,
            Some(fold),
            f,
        );
        f.instruction(&Instruction::I32Eqz);
        self.emit_temporal_provider_corruption_if_i32(f);
        prepared.identifier.set_reference(&identifier, schema, f);
        prepared.primary.set_reference(&primary, schema, f);
        f.instruction(&Instruction::I64Const(TemporalZoneKind::Named.code()));
        prepared.kind.store(f);
        self.emit_string_payload_equality_i32(&primary, utc, f);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(TemporalZoneKind::Utc.code()));
        prepared.kind.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        primary.clear(f);
        identifier.clear(f);
        schema.release_i64_local(start, f);
        schema.release_i64_local(primary_length, f);
        schema.release_i64_local(identifier_length, f);
        reply.clear(self, f);
        schema.release_i64_local(syntax, f);
        Ok(())
    }
    pub(in crate::builtins) fn emit_pinned_named_identifier_syntax(
        &mut self,
        input: &GcLocal<StringValue>,
        destination: I64Local,
        function: &mut Function,
    ) {
        let length = self.runtime_schema().reserve_i64_local(function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let component = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_string_length(input, length, function);
        function.instruction(&Instruction::I64Const(1));
        (destination).store(function);
        for slot in [cursor, component] {
            function.instruction(&Instruction::I64Const(0));
            (slot).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor).load(function);
        (length).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(input, cursor, byte, function);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'/' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (destination).load(function);
        (component).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        function.instruction(&Instruction::I64Const(0));
        (component).store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        for (start, end) in [(b'A', b'Z'), (b'a', b'z'), (b'0', b'9')] {
            (byte).load(function);
            function.instruction(&Instruction::I64Const(start as i64));
            function.instruction(&Instruction::I64GeU);
            (byte).load(function);
            function.instruction(&Instruction::I64Const(end as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
        }
        for allowed in [b'_', b'-', b'+'] {
            (byte).load(function);
            function.instruction(&Instruction::I64Const(allowed as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I64ExtendI32U);
        (destination).load(function);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        (component).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (component).store(function);
        (destination).load(function);
        (component).load(function);
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (cursor).load(function);
        function.instruction(&Instruction::I64Eqz);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        (destination).load(function);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        self.emit_temporal_advance_cursor(cursor, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (destination).load(function);
        (component).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (length).load(function);
        function.instruction(&Instruction::I64Const(
            MAX_TIME_ZONE_IDENTIFIER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        for slot in [byte, component, cursor, length] {
            self.runtime_schema().release_i64_local(slot, function);
        }
    }

    /// A raw IANA identifier and an ISO date/time string have different parse
    /// goals. Recognize the complete identifier grammar before trying ISO text;
    /// a prefix test would misclassify time strings beginning with `T`.
    pub(in crate::builtins) fn emit_temporal_named_identifier_syntax(
        &mut self,
        input: &GcLocal<StringValue>,
        destination: I64Local,
        function: &mut Function,
    ) {
        let length = self.runtime_schema().reserve_i64_local(function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        let component_head = self.runtime_schema().reserve_i64_local(function);
        let leading = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_string_length(input, length, function);
        function.instruction(&Instruction::I64Const(0));
        (cursor).store(function);
        function.instruction(&Instruction::I64Const(1));
        (destination).store(function);
        function.instruction(&Instruction::I64Const(1));
        (component_head).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor).load(function);
        (length).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(input, cursor, byte, function);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'/' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (destination).load(function);
        (component_head).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        function.instruction(&Instruction::I64Const(1));
        (component_head).store(function);
        function.instruction(&Instruction::Else);
        for (start, end) in [(b'A', b'Z'), (b'a', b'z')] {
            (byte).load(function);
            function.instruction(&Instruction::I64Const(start as i64));
            function.instruction(&Instruction::I64GeU);
            (byte).load(function);
            function.instruction(&Instruction::I64Const(end as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::I32Or);
        for allowed in [b'_', b'.'] {
            (byte).load(function);
            function.instruction(&Instruction::I64Const(allowed as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I64ExtendI32U);
        (leading).store(function);
        (leading).load(function);
        self.emit_temporal_byte_is_digit(byte, function);
        for allowed in [b'-', b'+'] {
            (byte).load(function);
            function.instruction(&Instruction::I64Const(allowed as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        (component_head).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Or);
        (destination).load(function);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        function.instruction(&Instruction::I64Const(0));
        (component_head).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (destination).load(function);
        (component_head).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        (destination).store(function);
        for local in [leading, component_head, byte, cursor, length] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
