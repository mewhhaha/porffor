//! Time-zone options are observed once; only completed canonical identity is retained.
use super::*;
pub(super) struct ResolvedDtfTimeZone {
    pub(super) identifier: GcLocal<StringValue>,
    pub(super) kind: GcI32DomainLocal<TimeZoneKind>,
    pub(super) fixed_seconds: I64Local,
}
impl ResolvedDtfTimeZone {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Result<Self, EmitError> {
        let schema = builder.runtime_schema();
        let identifier = schema
            .reserve_gc_local(f)
            .initialize(builder.emit_interned_string_reference("UTC", f)?, f);
        let kind = GcI32DomainLocal::new(schema, TimeZoneKind::Named, f);
        let fixed_seconds = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        fixed_seconds.store(f);
        Ok(Self {
            identifier,
            kind,
            fixed_seconds,
        })
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        schema.release_i64_local(self.fixed_seconds, f);
        self.kind.clear(schema, f);
        self.identifier.clear(f);
    }
    pub(super) fn append(
        &self,
        message: &IntlByteArrayBuilder,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        dtf_append_domain(message, &self.kind, schema, f);
        emit_domain_is(&self.kind, TimeZoneKind::Named, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        message.append_utf8(&self.identifier, schema, f);
        f.instruction(&Instruction::Else);
        message.append_u64(self.fixed_seconds, schema, f);
        f.instruction(&Instruction::End);
    }
}
fn corrupt_if(f: &mut Function) {
    f.instruction(&Instruction::If(BlockType::Empty));
    f.instruction(&Instruction::Unreachable);
    f.instruction(&Instruction::End);
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_time_zone_option(
        &mut self,
        options: &ValueLocals,
        forced: Option<&ResolvedDtfTimeZone>,
        f: &mut Function,
    ) -> Result<ResolvedDtfTimeZone, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_intl_number_get_option(options, "timeZone", &value, f)?;
        let zone = ResolvedDtfTimeZone::new(self, f)?;
        if let Some(forced) = forced {
            emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
            f.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_intl_number_type_error(RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROTOTYPE_TOLOCALESTRING_DOES_NOT_SUPPORT_THE_TIMEZONE_OPTION,f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            zone.identifier
                .replace(forced.identifier.load(schema, f), f);
            zone.kind.copy_from(&forced.kind, f);
            forced.fixed_seconds.load(f);
            zone.fixed_seconds.store(f);
        } else {
            emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
            self.open_frame(ControlFrameKind::If, f);
            let system = self.emit_system_time_zone(f)?;
            zone.identifier
                .replace(system.identifier().load(schema, f), f);
            system.fixed_offset_seconds().load(f);
            zone.fixed_seconds.store(f);
            for kind in [
                SystemTimeZoneKind::Utc,
                SystemTimeZoneKind::FixedOffset,
                SystemTimeZoneKind::Named,
            ] {
                system.kind_local().load(f);
                f.instruction(&Instruction::I64Const(kind.code()));
                f.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, f);
                zone.kind.set_constant(
                    match kind {
                        SystemTimeZoneKind::FixedOffset => TimeZoneKind::FixedOffset,
                        SystemTimeZoneKind::Utc | SystemTimeZoneKind::Named => TimeZoneKind::Named,
                    },
                    f,
                );
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            system.release(self, f);
            f.instruction(&Instruction::Else);
            let text = self.emit_intl_number_to_string(&value, f)?;
            self.emit_dtf_resolve_zone_string(&text, &zone, false, f)?;
            text.clear(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        value.clear(f);
        Ok(zone)
    }
    pub(super) fn emit_dtf_resolve_zone_string(
        &mut self,
        text: &GcLocal<StringValue>,
        zone: &ResolvedDtfTimeZone,
        primary: bool,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let minutes = schema.reserve_i64_local(f);
        let accepted = schema.reserve_i32_local(f);
        self.emit_dtf_parse_offset(text, minutes, accepted, f);
        accepted.load(f);
        self.open_frame(ControlFrameKind::If, f);
        zone.kind.set_constant(TimeZoneKind::FixedOffset, f);
        minutes.load(f);
        f.instruction(&Instruction::I64Const(60));
        f.instruction(&Instruction::I64Mul);
        zone.fixed_seconds.store(f);
        zone.identifier
            .replace(self.emit_dtf_offset_identifier(minutes, f), f);
        f.instruction(&Instruction::Else);
        zone.kind.set_constant(TimeZoneKind::Named, f);
        let named = self.emit_dtf_lookup_named_zone(text, primary, f)?;
        zone.identifier.replace(named.load(schema, f), f);
        named.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(accepted, f);
        schema.release_i64_local(minutes, f);
        Ok(())
    }
    fn emit_dtf_parse_offset(
        &self,
        text: &GcLocal<StringValue>,
        minutes: I64Local,
        accepted: I32Local,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(text, schema, f)
                .reference(),
            f,
        );
        let length = schema.reserve_i32_local(f);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, f);
        length.store(f);
        let index = schema.reserve_i32_local(f);
        let unit = schema.reserve_i32_local(f);
        let sign = schema.reserve_i64_local(f);
        let hour = schema.reserve_i64_local(f);
        let minute = schema.reserve_i64_local(f);
        let minute_start = schema.reserve_i32_local(f);
        set_i32(accepted, 0, f);
        set_i32(minute_start, 0, f);
        for value in [minutes, hour, minute] {
            f.instruction(&Instruction::I64Const(0));
            value.store(f);
        }
        f.instruction(&Instruction::Block(BlockType::Empty));
        for (i, size) in [3, 5, 6].into_iter().enumerate() {
            length.load(f);
            f.instruction(&Instruction::I32Const(size));
            f.instruction(&Instruction::I32Eq);
            if i != 0 {
                f.instruction(&Instruction::I32Or);
            }
        }
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(0));
        set_i32(index, 0, f);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, f)
            .store(unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(43));
        f.instruction(&Instruction::I32Eq);
        unit.load(f);
        f.instruction(&Instruction::I32Const(45));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(0));
        unit.load(f);
        f.instruction(&Instruction::I32Const(45));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::End);
        sign.store(f);
        for position in [1, 2] {
            set_i32(index, position, f);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, f)
                .store(unit, f);
            self.emit_dtf_offset_digit(unit, 0, f);
            hour.load(f);
            f.instruction(&Instruction::I64Const(10));
            f.instruction(&Instruction::I64Mul);
            unit.load(f);
            f.instruction(&Instruction::I64ExtendI32U);
            f.instruction(&Instruction::I64Const(48));
            f.instruction(&Instruction::I64Sub);
            f.instruction(&Instruction::I64Add);
            hour.store(f);
        }
        hour.load(f);
        f.instruction(&Instruction::I64Const(FixedTimeZoneOffset::MAX_HOUR));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::BrIf(0));
        length.load(f);
        f.instruction(&Instruction::I32Const(5));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        set_i32(minute_start, 3, f);
        f.instruction(&Instruction::End);
        length.load(f);
        f.instruction(&Instruction::I32Const(6));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        set_i32(index, 3, f);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, f)
            .store(unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(58));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        set_i32(minute_start, 4, f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        length.load(f);
        f.instruction(&Instruction::I32Const(3));
        f.instruction(&Instruction::I32Ne);
        minute_start.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::BrIf(0));
        minute_start.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        for delta in [0, 1] {
            minute_start.load(f);
            f.instruction(&Instruction::I32Const(delta));
            f.instruction(&Instruction::I32Add);
            index.store(f);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, f)
                .store(unit, f);
            self.emit_dtf_offset_digit(unit, 1, f);
            minute.load(f);
            f.instruction(&Instruction::I64Const(10));
            f.instruction(&Instruction::I64Mul);
            unit.load(f);
            f.instruction(&Instruction::I64ExtendI32U);
            f.instruction(&Instruction::I64Const(48));
            f.instruction(&Instruction::I64Sub);
            f.instruction(&Instruction::I64Add);
            minute.store(f);
        }
        minute.load(f);
        f.instruction(&Instruction::I64Const(FixedTimeZoneOffset::MAX_MINUTE));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::BrIf(1));
        f.instruction(&Instruction::End);
        hour.load(f);
        f.instruction(&Instruction::I64Const(60));
        f.instruction(&Instruction::I64Mul);
        minute.load(f);
        f.instruction(&Instruction::I64Add);
        sign.load(f);
        f.instruction(&Instruction::I64Mul);
        minutes.store(f);
        set_i32(accepted, 1, f);
        f.instruction(&Instruction::End);
        schema.release_i32_local(minute_start, f);
        schema.release_i64_local(minute, f);
        schema.release_i64_local(hour, f);
        schema.release_i64_local(sign, f);
        schema.release_i32_local(unit, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(length, f);
        units.clear(f);
    }
    fn emit_dtf_offset_digit(&self, unit: I32Local, depth: u32, f: &mut Function) {
        unit.load(f);
        f.instruction(&Instruction::I32Const(48));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(9));
        f.instruction(&Instruction::I32GtU);
        f.instruction(&Instruction::BrIf(depth));
    }
    fn emit_dtf_offset_identifier(
        &self,
        minutes: I64Local,
        f: &mut Function,
    ) -> GcStackReference<StringValue> {
        let schema = self.runtime_schema();
        let count = schema.reserve_i32_local(f);
        set_i32(count, 6, f);
        let construction =
            StringConstruction::allocate(schema, schema.reserve_gc_local(f), count, f);
        let index = schema.reserve_i32_local(f);
        let unit = schema.reserve_i32_local(f);
        let magnitude = schema.reserve_i64_local(f);
        minutes.load(f);
        f.instruction(&Instruction::I64Const(0));
        f.instruction(&Instruction::I64LtS);
        f.instruction(&Instruction::If(BlockType::Empty));
        set_i32(unit, 45, f);
        f.instruction(&Instruction::I64Const(0));
        minutes.load(f);
        f.instruction(&Instruction::I64Sub);
        magnitude.store(f);
        f.instruction(&Instruction::Else);
        set_i32(unit, 43, f);
        minutes.load(f);
        magnitude.store(f);
        f.instruction(&Instruction::End);
        set_i32(index, 0, f);
        construction.write(index, unit, schema, f);
        for (position, divisor) in [(1, 600), (2, 60), (4, 10), (5, 1)] {
            set_i32(index, position, f);
            magnitude.load(f);
            if position < 3 {
                f.instruction(&Instruction::I64Const(60));
                f.instruction(&Instruction::I64DivU);
                if divisor == 600 {
                    f.instruction(&Instruction::I64Const(10));
                    f.instruction(&Instruction::I64DivU);
                } else {
                    f.instruction(&Instruction::I64Const(10));
                    f.instruction(&Instruction::I64RemU);
                }
            } else {
                f.instruction(&Instruction::I64Const(60));
                f.instruction(&Instruction::I64RemU);
                f.instruction(&Instruction::I64Const(10));
                if divisor == 10 {
                    f.instruction(&Instruction::I64DivU);
                } else {
                    f.instruction(&Instruction::I64RemU);
                }
            }
            f.instruction(&Instruction::I32WrapI64);
            f.instruction(&Instruction::I32Const(48));
            f.instruction(&Instruction::I32Add);
            unit.store(f);
            construction.write(index, unit, schema, f);
        }
        set_i32(index, 3, f);
        set_i32(unit, 58, f);
        construction.write(index, unit, schema, f);
        schema.release_i64_local(magnitude, f);
        schema.release_i32_local(unit, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        construction.publish(schema, f)
    }
    fn emit_dtf_read_zone_identifier(
        &self,
        reader: &IntlByteArrayReader<'_>,
        length: I64Local,
        f: &mut Function,
    ) -> GcLocal<StringValue> {
        let schema = self.runtime_schema();
        reader.require_records(length, 1, f);
        length.load(f);
        f.instruction(&Instruction::I64Eqz);
        length.load(f);
        f.instruction(&Instruction::I64Const(
            lila_intl::MAX_TIME_ZONE_IDENTIFIER_BYTES as i64,
        ));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        corrupt_if(f);
        let count = schema.reserve_i32_local(f);
        length.load(f);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let construction =
            StringConstruction::allocate(schema, schema.reserve_gc_local(f), count, f);
        let index = schema.reserve_i32_local(f);
        let unit = schema.reserve_i32_local(f);
        set_i32(index, 0, f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        f.instruction(&Instruction::BrIf(1));
        reader.read_u8(unit, schema, f);
        unit.load(f);
        f.instruction(&Instruction::I32Eqz);
        unit.load(f);
        f.instruction(&Instruction::I32Const(127));
        f.instruction(&Instruction::I32GtU);
        f.instruction(&Instruction::I32Or);
        corrupt_if(f);
        construction.write(index, unit, schema, f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        let result = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(schema, f), f);
        schema.release_i32_local(unit, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        result
    }
    fn emit_dtf_lookup_named_zone(
        &mut self,
        text: &GcLocal<StringValue>,
        select_primary: bool,
        f: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let message =
            IntlByteArrayBuilder::with_operation(IntlHostOp::LookupNamedTimeZone, schema, f);
        message.append_remaining_utf8(text, schema, f);
        let request = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&request, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let bytes = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        request.clear(f);
        let reader = IntlByteArrayReader::new(&bytes, schema, f);
        let identifier_length = schema.reserve_i64_local(f);
        let primary_length = schema.reserve_i64_local(f);
        reader.read_u64(identifier_length, schema, f);
        reader.read_u64(primary_length, schema, f);
        let identifier = self.emit_dtf_read_zone_identifier(&reader, identifier_length, f);
        let primary = self.emit_dtf_read_zone_identifier(&reader, primary_length, f);
        reader.finish(schema, f);
        schema.release_i64_local(primary_length, f);
        schema.release_i64_local(identifier_length, f);
        bytes.clear(f);
        if select_primary {
            identifier.clear(f);
            Ok(primary)
        } else {
            primary.clear(f);
            Ok(identifier)
        }
    }
    pub(super) fn emit_dtf_read_selected_zone(
        &mut self,
        reader: &IntlByteArrayReader<'_>,
        requested: &ResolvedDtfTimeZone,
        f: &mut Function,
    ) -> ResolvedDtfTimeZone {
        let schema = self.runtime_schema();
        let identifier = schema
            .reserve_gc_local(f)
            .initialize(requested.identifier.load(schema, f), f);
        let kind = GcI32DomainLocal::new(schema, TimeZoneKind::Named, f);
        self.emit_dtf_read_domain(
            reader,
            &kind,
            [
                (TimeZoneKind::Named, TimeZoneKind::Named.code() as u64),
                (
                    TimeZoneKind::FixedOffset,
                    TimeZoneKind::FixedOffset.code() as u64,
                ),
            ],
            f,
        );
        let fixed_seconds = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        fixed_seconds.store(f);
        emit_domain_is(&kind, TimeZoneKind::Named, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        let name = reader.read_utf8(schema, f);
        self.emit_string_payload_equality_i32(&name, &requested.identifier, f);
        f.instruction(&Instruction::I32Eqz);
        corrupt_if(f);
        identifier.replace(name.load(schema, f), f);
        name.clear(f);
        f.instruction(&Instruction::Else);
        reader.read_u64(fixed_seconds, schema, f);
        fixed_seconds.load(f);
        requested.fixed_seconds.load(f);
        f.instruction(&Instruction::I64Ne);
        corrupt_if(f);
        f.instruction(&Instruction::End);
        kind.load(f);
        requested.kind.load(f);
        f.instruction(&Instruction::I32Ne);
        corrupt_if(f);
        ResolvedDtfTimeZone {
            identifier,
            kind,
            fixed_seconds,
        }
    }
}
