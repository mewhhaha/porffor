//! ZonedDateTime locale formatting uses its retained GC slots and an intrinsic service.
use super::*;
use provider_render::DtfFormatMode;
impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_zoned_component_defaults(
        &self,
        components: &DtfComponentsLocals,
        f: &mut Function,
    ) {
        // Required(any) excludes era and timeZoneName. Their original getters ran.
        emit_domain_is(&components.weekday, None, f);
        emit_domain_is(&components.year, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.month, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.day, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.day_period, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.hour, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.minute, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.second, None, f);
        f.instruction(&Instruction::I32And);
        emit_domain_is(&components.fractional, None, f);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        emit_domain_is(&components.zone_name, None, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        components
            .zone_name
            .set_constant(Some(TimeZoneNameStyle::Short), f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
    }
    pub(crate) fn emit_intl_dtf_format_temporal_zoned(
        &mut self,
        zoned: &GcLocal<TemporalZonedDateTimeObject>,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = schema.struct_type::<TemporalZonedDateTimeObject>();
        let zone_text = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalZonedDateTimeObjectSchema::TIME_ZONE)
                .read(zoned, schema, f)
                .reference(),
            f,
        );
        let zone = ResolvedDtfTimeZone::new(self, f)?;
        self.emit_dtf_resolve_zone_string(&zone_text, &zone, true, f)?;
        zone_text.clear(f);
        let header = self.emit_reserve_intrinsic_date_time_format_object(f)?;
        let formatter = self.emit_dtf_complete_initialization(
            header,
            &IntlDateTimeFormatPurpose::Temporal(DtfTemporalKind::Instant),
            locales,
            options,
            Some(&zone),
            f,
        )?;
        let calendar = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalZonedDateTimeObjectSchema::CALENDAR)
                .read(zoned, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_calendar_compatibility(&formatter, &calendar, true, f)?;
        calendar.clear(f);
        let epoch = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalZonedDateTimeObjectSchema::EPOCH_NANOSECONDS)
                .read(zoned, schema, f)
                .reference(),
            f,
        );
        let input = CompletedDtfInput::new(schema, f);
        f.instruction(&Instruction::I64Const(
            DateTimeValueKind::Instant.wire_code() as i64,
        ));
        input.words[0].store(f);
        self.emit_dtf_epoch_pair(&epoch, input.words[1], input.words[2], f);
        epoch.clear(f);
        // Intl's maximum fractional precision is three digits. Negative epoch pairs
        // already use floor seconds and a positive remainder, so -1ns becomes -1ms.
        input.words[2].load(f);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64Mul);
        input.words[2].store(f);
        let out = schema.reserve_value_local(f);
        self.emit_dtf_provider_format(&formatter, &input, None, DtfFormatMode::String, &out, f)?;
        self.completion().initialize(f);
        self.completion().value().copy_from(&out, f);
        out.clear(f);
        input.clear(schema, f);
        formatter.clear(f);
        zone.clear(schema, f);
        Ok(())
    }
}
