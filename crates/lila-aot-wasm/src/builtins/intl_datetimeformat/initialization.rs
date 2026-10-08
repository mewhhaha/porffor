use super::*;
use crate::builtins::date::DateLocaleFormat;
use construction_lifecycle::ReservedIntlDateTimeFormatObjectLocal;
pub(crate) enum IntlDateTimeFormatPurpose {
    Constructor,
    DateLocale(DateLocaleFormat),
    Temporal(DtfTemporalKind),
}
#[derive(Clone, Copy)]
pub(super) enum RejectedDateTimeStyle {
    Date,
    Time,
}
impl IntlDateTimeFormatPurpose {
    fn required(&self) -> DateTimeRequired {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::DateAndTime) => {
                DateTimeRequired::Any
            }
            Self::DateLocale(DateLocaleFormat::Date) => DateTimeRequired::Date,
            Self::DateLocale(DateLocaleFormat::Time) => DateTimeRequired::Time,
            Self::Temporal(kind) => match kind {
                DtfTemporalKind::PlainDate
                | DtfTemporalKind::PlainYearMonth
                | DtfTemporalKind::PlainMonthDay => DateTimeRequired::Date,
                DtfTemporalKind::PlainTime => DateTimeRequired::Time,
                DtfTemporalKind::PlainDateTime | DtfTemporalKind::Instant => DateTimeRequired::Any,
            },
        }
    }
    fn defaults(&self) -> DateTimeDefaults {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::Date) => DateTimeDefaults::Date,
            Self::DateLocale(DateLocaleFormat::Time) => DateTimeDefaults::Time,
            Self::DateLocale(DateLocaleFormat::DateAndTime) => DateTimeDefaults::All,
            Self::Temporal(kind) => match kind {
                DtfTemporalKind::PlainDate
                | DtfTemporalKind::PlainYearMonth
                | DtfTemporalKind::PlainMonthDay => DateTimeDefaults::Date,
                DtfTemporalKind::PlainTime => DateTimeDefaults::Time,
                DtfTemporalKind::PlainDateTime | DtfTemporalKind::Instant => DateTimeDefaults::All,
            },
        }
    }
    fn rejected_style(&self) -> Option<(RejectedDateTimeStyle, RuntimeErrorMessage)> {
        match self{Self::Constructor|Self::DateLocale(DateLocaleFormat::DateAndTime)=>None,
 Self::DateLocale(DateLocaleFormat::Date)=>Some((RejectedDateTimeStyle::Time,RuntimeErrorMessage::DATE_PROTOTYPE_TOLOCALEDATESTRING_DOES_NOT_SUPPORT_THE_TIMESTYLE_OPTION)),
 Self::DateLocale(DateLocaleFormat::Time)=>Some((RejectedDateTimeStyle::Date,RuntimeErrorMessage::DATE_PROTOTYPE_TOLOCALETIMESTRING_DOES_NOT_SUPPORT_THE_DATESTYLE_OPTION)),Self::Temporal(kind)=>kind.rejected_style()}
    }
}
pub(super) struct DtfComponentsLocals {
    pub(super) weekday: GcI32DomainLocal<Option<DateTimeTextWidth>>,
    pub(super) era: GcI32DomainLocal<Option<DateTimeTextWidth>>,
    pub(super) year: GcI32DomainLocal<Option<DateTimeNumericWidth>>,
    pub(super) month: GcI32DomainLocal<Option<DateTimeMonthWidth>>,
    pub(super) day: GcI32DomainLocal<Option<DateTimeNumericWidth>>,
    pub(super) day_period: GcI32DomainLocal<Option<DateTimeTextWidth>>,
    pub(super) hour: GcI32DomainLocal<Option<DateTimeNumericWidth>>,
    pub(super) minute: GcI32DomainLocal<Option<DateTimeNumericWidth>>,
    pub(super) second: GcI32DomainLocal<Option<DateTimeNumericWidth>>,
    pub(super) fractional: GcI32DomainLocal<Option<DateTimeFractionalDigits>>,
    pub(super) zone_name: GcI32DomainLocal<Option<TimeZoneNameStyle>>,
    pub(super) date_style: GcI32DomainLocal<Option<DateTimeStyle>>,
    pub(super) time_style: GcI32DomainLocal<Option<DateTimeStyle>>,
}
impl DtfComponentsLocals {
    pub(super) fn new(schema: &RuntimeSchema, f: &mut Function) -> Self {
        Self {
            weekday: GcI32DomainLocal::new(schema, None::<DateTimeTextWidth>, f),
            era: GcI32DomainLocal::new(schema, None::<DateTimeTextWidth>, f),
            year: GcI32DomainLocal::new(schema, None::<DateTimeNumericWidth>, f),
            month: GcI32DomainLocal::new(schema, None::<DateTimeMonthWidth>, f),
            day: GcI32DomainLocal::new(schema, None::<DateTimeNumericWidth>, f),
            day_period: GcI32DomainLocal::new(schema, None::<DateTimeTextWidth>, f),
            hour: GcI32DomainLocal::new(schema, None::<DateTimeNumericWidth>, f),
            minute: GcI32DomainLocal::new(schema, None::<DateTimeNumericWidth>, f),
            second: GcI32DomainLocal::new(schema, None::<DateTimeNumericWidth>, f),
            fractional: GcI32DomainLocal::new(schema, None::<DateTimeFractionalDigits>, f),
            zone_name: GcI32DomainLocal::new(schema, None::<TimeZoneNameStyle>, f),
            date_style: GcI32DomainLocal::new(schema, None::<DateTimeStyle>, f),
            time_style: GcI32DomainLocal::new(schema, None::<DateTimeStyle>, f),
        }
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        self.time_style.clear(schema, f);
        self.date_style.clear(schema, f);
        self.zone_name.clear(schema, f);
        self.fractional.clear(schema, f);
        self.second.clear(schema, f);
        self.minute.clear(schema, f);
        self.hour.clear(schema, f);
        self.day_period.clear(schema, f);
        self.day.clear(schema, f);
        self.month.clear(schema, f);
        self.year.clear(schema, f);
        self.era.clear(schema, f);
        self.weekday.clear(schema, f);
    }
    pub(super) fn read_record(
        &self,
        record: &GcLocal<IntlDateTimeFormatObject>,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        let dtf = schema.struct_type::<IntlDateTimeFormatObject>();
        dtf.field(IntlDateTimeFormatObjectSchema::WEEKDAY)
            .read(record, schema, f)
            .store_domain(&self.weekday, f);
        dtf.field(IntlDateTimeFormatObjectSchema::ERA)
            .read(record, schema, f)
            .store_domain(&self.era, f);
        dtf.field(IntlDateTimeFormatObjectSchema::YEAR)
            .read(record, schema, f)
            .store_domain(&self.year, f);
        dtf.field(IntlDateTimeFormatObjectSchema::MONTH)
            .read(record, schema, f)
            .store_domain(&self.month, f);
        dtf.field(IntlDateTimeFormatObjectSchema::DAY)
            .read(record, schema, f)
            .store_domain(&self.day, f);
        dtf.field(IntlDateTimeFormatObjectSchema::DAY_PERIOD)
            .read(record, schema, f)
            .store_domain(&self.day_period, f);
        dtf.field(IntlDateTimeFormatObjectSchema::HOUR)
            .read(record, schema, f)
            .store_domain(&self.hour, f);
        dtf.field(IntlDateTimeFormatObjectSchema::MINUTE)
            .read(record, schema, f)
            .store_domain(&self.minute, f);
        dtf.field(IntlDateTimeFormatObjectSchema::SECOND)
            .read(record, schema, f)
            .store_domain(&self.second, f);
        dtf.field(IntlDateTimeFormatObjectSchema::FRACTIONAL_SECOND_DIGITS)
            .read(record, schema, f)
            .store_domain(&self.fractional, f);
        dtf.field(IntlDateTimeFormatObjectSchema::TIME_ZONE_NAME)
            .read(record, schema, f)
            .store_domain(&self.zone_name, f);
        dtf.field(IntlDateTimeFormatObjectSchema::DATE_STYLE)
            .read(record, schema, f)
            .store_domain(&self.date_style, f);
        dtf.field(IntlDateTimeFormatObjectSchema::TIME_STYLE)
            .read(record, schema, f)
            .store_domain(&self.time_style, f);
    }
    pub(super) fn append_components(
        &self,
        message: &IntlByteArrayBuilder,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        dtf_append_optional(message, &self.weekday, schema, f);
        dtf_append_optional(message, &self.era, schema, f);
        dtf_append_optional(message, &self.year, schema, f);
        dtf_append_optional(message, &self.month, schema, f);
        dtf_append_optional(message, &self.day, schema, f);
        dtf_append_optional(message, &self.day_period, schema, f);
        dtf_append_optional(message, &self.hour, schema, f);
        dtf_append_optional(message, &self.minute, schema, f);
        dtf_append_optional(message, &self.second, schema, f);
        dtf_append_optional(message, &self.fractional, schema, f);
        dtf_append_optional(message, &self.zone_name, schema, f);
    }
    pub(super) fn any_component(&self, f: &mut Function) {
        emit_domain_is(&self.weekday, None, f);
        f.instruction(&Instruction::I32Eqz);
        emit_domain_is(&self.era, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.year, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.month, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.day, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.day_period, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.hour, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.minute, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.second, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.fractional, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        emit_domain_is(&self.zone_name, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
    }
}
pub(super) struct CompletedDtfLocale {
    locale: GcLocal<StringValue>,
    data_locale: GcLocal<StringValue>,
    calendar: GcI32DomainLocal<DateTimeCalendar>,
    numbering: GcLocal<StringValue>,
    hour_cycle: GcI32DomainLocal<DateTimeHourCycle>,
}
impl CompletedDtfLocale {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        self.hour_cycle.clear(schema, f);
        self.numbering.clear(f);
        self.calendar.clear(schema, f);
        self.data_locale.clear(f);
        self.locale.clear(f);
    }
    fn append(&self, message: &IntlByteArrayBuilder, schema: &RuntimeSchema, f: &mut Function) {
        message.append_utf8(&self.locale, schema, f);
        message.append_utf8(&self.data_locale, schema, f);
        dtf_append_domain(message, &self.calendar, schema, f);
        message.append_utf8(&self.numbering, schema, f);
        dtf_append_domain(message, &self.hour_cycle, schema, f);
    }
}
impl FunctionBuilder<'_> {
    fn emit_dtf_read_locale(
        &self,
        reader: &IntlByteArrayReader<'_>,
        f: &mut Function,
    ) -> CompletedDtfLocale {
        let schema = self.runtime_schema();
        let locale = reader.read_utf8(schema, f);
        let data_locale = reader.read_utf8(schema, f);
        let calendar = GcI32DomainLocal::new(schema, DateTimeCalendar::Gregorian, f);
        self.emit_dtf_read_domain(
            reader,
            &calendar,
            DateTimeCalendar::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        let numbering = reader.read_utf8(schema, f);
        let hour_cycle = GcI32DomainLocal::new(schema, DateTimeHourCycle::H23, f);
        self.emit_dtf_read_domain(
            reader,
            &hour_cycle,
            DateTimeHourCycle::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        CompletedDtfLocale {
            locale,
            data_locale,
            calendar,
            numbering,
            hour_cycle,
        }
    }
    fn emit_dtf_keyword_option(
        &mut self,
        options: &ValueLocals,
        property: IntlErrorOption,
        f: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_intl_number_string_value(options, property.property(), &value, f)?;
        let out = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("", f)?, f);
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let text = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(schema, f), f);
        let valid = schema.reserve_i32_local(f);
        self.emit_intl_is_unicode_type_i32(&text, valid, f);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(property.error_message(), f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        out.replace(self.emit_dtf_ascii_lowercase(&text, f), f);
        schema.release_i32_local(valid, f);
        text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.clear(f);
        Ok(out)
    }
    fn emit_dtf_ascii_lowercase(
        &self,
        text: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> GcStackReference<StringValue> {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(text, schema, f)
                .reference(),
            f,
        );
        let count = schema.reserve_i32_local(f);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, f);
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
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, f)
            .store(unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(65));
        f.instruction(&Instruction::I32GeU);
        unit.load(f);
        f.instruction(&Instruction::I32Const(90));
        f.instruction(&Instruction::I32LeU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        unit.load(f);
        f.instruction(&Instruction::I32Const(32));
        f.instruction(&Instruction::I32Add);
        unit.store(f);
        f.instruction(&Instruction::End);
        construction.write(index, unit, schema, f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        schema.release_i32_local(unit, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        units.clear(f);
        construction.publish(schema, f)
    }
    fn emit_dtf_observe_components(
        &mut self,
        options: &ValueLocals,
        components: &DtfComponentsLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Weekday,
            DateTimeTextWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.weekday,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Era,
            DateTimeTextWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.era,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Year,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.year,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Month,
            DateTimeMonthWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.month,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Day,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.day,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::DayPeriod,
            DateTimeTextWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.day_period,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Hour,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.hour,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Minute,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.minute,
            f,
        )?;
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Second,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.second,
            f,
        )?;
        let value = schema.reserve_value_local(f);
        self.emit_intl_number_get_option(options, "fractionalSecondDigits", &value, f)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let fallback = schema.reserve_i32_local(f);
        let count = schema.reserve_i32_local(f);
        set_i32(fallback, 1, f);
        self.emit_intl_number_coerce_digit(
            &value,
            IntlErrorOption::FractionalSecondDigits,
            1,
            3,
            fallback,
            count,
            f,
        )?;
        for digit in 1..=3 {
            count.load(f);
            f.instruction(&Instruction::I32Const(digit));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            components.fractional.set_constant(
                Some(DateTimeFractionalDigits::new(digit as u8).expect("closed fractional digits")),
                f,
            );
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        schema.release_i32_local(count, f);
        schema.release_i32_local(fallback, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.clear(f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::TimeZoneName,
            TimeZoneNameStyle::ALL
                .into_iter()
                .map(|v| (v.spelling(), Some(v))),
            None,
            &components.zone_name,
            f,
        )?;
        Ok(())
    }
    pub(crate) fn emit_intl_create_date_time_format(
        &mut self,
        purpose: IntlDateTimeFormatPurpose,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let header = match purpose {
            IntlDateTimeFormatPurpose::Constructor => {
                self.emit_reserve_intl_date_time_format_object(f)?
            }
            IntlDateTimeFormatPurpose::DateLocale(_) | IntlDateTimeFormatPurpose::Temporal(_) => {
                self.emit_reserve_intrinsic_date_time_format_object(f)?
            }
        };
        let schema = self.runtime_schema();
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let record =
            self.emit_dtf_complete_initialization(header, &purpose, &locales, &options, None, f)?;
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
    pub(super) fn emit_dtf_complete_initialization(
        &mut self,
        header: ReservedIntlDateTimeFormatObjectLocal,
        purpose: &IntlDateTimeFormatPurpose,
        locales: &ValueLocals,
        original_options: &ValueLocals,
        forced_zone: Option<&ResolvedDtfTimeZone>,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDateTimeFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let requested = self.emit_intl_canonical_locale_list(locales, f)?;
        let options = schema.reserve_value_local(f);
        options.copy_from(original_options, f);
        self.emit_intl_number_options_object(&options, f)?;
        let matcher = GcI32DomainLocal::new(schema, DateTimeLocaleMatcher::BestFit, f);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::LocaleMatcher,
            DateTimeLocaleMatcher::ALL
                .iter()
                .map(|v| (v.option_name(), *v)),
            DateTimeLocaleMatcher::BestFit,
            &matcher,
            f,
        )?;
        let calendar = self.emit_dtf_keyword_option(&options, IntlErrorOption::Calendar, f)?;
        let numbering =
            self.emit_dtf_keyword_option(&options, IntlErrorOption::NumberingSystem, f)?;
        let observed = schema.reserve_value_local(f);
        self.emit_intl_number_get_option(&options, "hour12", &observed, f)?;
        let hour12 = GcI32DomainLocal::new(schema, None::<bool>, f);
        emit_tag_is(&observed, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.compile_truthy_tagged_i32(&observed, f)?;
        self.open_frame(ControlFrameKind::If, f);
        hour12.set_constant(Some(true), f);
        f.instruction(&Instruction::Else);
        hour12.set_constant(Some(false), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let requested_cycle = GcI32DomainLocal::new(schema, None::<DateTimeHourCycle>, f);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::HourCycle,
            DateTimeHourCycle::ALL
                .iter()
                .map(|v| (v.as_str(), Some(*v))),
            None,
            &requested_cycle,
            f,
        )?;
        let preference = GcI32DomainLocal::new(schema, DateTimeHourCyclePreference::Default, f);
        for cycle in DateTimeHourCycle::ALL {
            emit_domain_is(&requested_cycle, Some(*cycle), f);
            self.open_frame(ControlFrameKind::If, f);
            preference.set_constant(DateTimeHourCyclePreference::Cycle(*cycle), f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        for (value, selected) in [
            (true, DateTimeHourCyclePreference::TwelveHour),
            (false, DateTimeHourCyclePreference::TwentyFourHour),
        ] {
            emit_domain_is(&hour12, Some(value), f);
            self.open_frame(ControlFrameKind::If, f);
            preference.set_constant(selected, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let message = self.emit_dtf_request(IntlHostOp::ResolveDateTimeLocale, f);
        dtf_append_domain(&message, &matcher, schema, f);
        dtf_append_domain(&message, &preference, schema, f);
        message.append_utf8(&calendar, schema, f);
        message.append_utf8(&numbering, schema, f);
        self.emit_intl_wire_canonical_locales(&message, &requested, f)?;
        let response =
            self.emit_dtf_provider_call(IntlHostOp::ResolveDateTimeLocale, message, f)?;
        let reader = response.reader(schema, f);
        let resolved = self.emit_dtf_read_locale(&reader, f);
        reader.finish(schema, f);
        response.clear(f);
        let zone = self.emit_dtf_time_zone_option(&options, forced_zone, f)?;
        let components = DtfComponentsLocals::new(schema, f);
        self.emit_dtf_observe_components(&options, &components, f)?;
        let format_matcher = GcI32DomainLocal::new(schema, DateTimeFormatMatcher::BestFit, f);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::FormatMatcher,
            DateTimeFormatMatcher::ALL
                .iter()
                .map(|v| (v.option_name(), *v)),
            DateTimeFormatMatcher::BestFit,
            &format_matcher,
            f,
        )?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::DateStyle,
            DateTimeStyle::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.date_style,
            f,
        )?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::TimeStyle,
            DateTimeStyle::ALL
                .iter()
                .map(|v| (v.option_name(), Some(*v))),
            None,
            &components.time_style,
            f,
        )?;
        emit_domain_is(&components.date_style, None, f);
        emit_domain_is(&components.time_style, None, f);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Eqz);
        components.any_component(f);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RuntimeErrorMessage::DATESTYLE_AND_TIMESTYLE_MAY_NOT_BE_USED_WITH_EXPLICIT_DATE_TIME_COMPONENTS,f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        if let Some((style, message)) = purpose.rejected_style() {
            emit_domain_is(
                match style {
                    RejectedDateTimeStyle::Date => &components.date_style,
                    RejectedDateTimeStyle::Time => &components.time_style,
                },
                None,
                f,
            );
            f.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_intl_number_type_error(message, f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let message = self.emit_dtf_request(IntlHostOp::SelectDateTimeFormat, f);
        resolved.append(&message, schema, f);
        zone.append(&message, schema, f);
        dtf_append_domain(&message, &format_matcher, schema, f);
        message.append_u64_constant(purpose.required().wire_code(), schema, f);
        message.append_u64_constant(purpose.defaults().wire_code(), schema, f);
        emit_domain_is(&components.date_style, None, f);
        emit_domain_is(&components.time_style, None, f);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        if forced_zone.is_some() {
            self.emit_dtf_zoned_component_defaults(&components, f);
        }
        message.append_u64_constant(1, schema, f);
        components.append_components(&message, schema, f);
        f.instruction(&Instruction::Else);
        message.append_u64_constant(2, schema, f);
        dtf_append_optional(&message, &components.date_style, schema, f);
        dtf_append_optional(&message, &components.time_style, schema, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let response = self.emit_dtf_provider_call(IntlHostOp::SelectDateTimeFormat, message, f)?;
        let record = self.emit_dtf_publish_selected_plan(header, &zone, &response, f)?;
        response.clear(f);
        format_matcher.clear(schema, f);
        components.clear(schema, f);
        zone.clear(schema, f);
        resolved.clear(schema, f);
        preference.clear(schema, f);
        requested_cycle.clear(schema, f);
        hour12.clear(schema, f);
        observed.clear(f);
        numbering.clear(f);
        calendar.clear(f);
        matcher.clear(schema, f);
        options.clear(f);
        requested.clear(f);
        Ok(record)
    }
    fn emit_dtf_publish_selected_plan(
        &mut self,
        header: ReservedIntlDateTimeFormatObjectLocal,
        requested_zone: &ResolvedDtfTimeZone,
        response: &DtfProviderResponse,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDateTimeFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let reader = response.reader(schema, f);
        let locale = self.emit_dtf_read_locale(&reader, f);
        let zone = self.emit_dtf_read_selected_zone(&reader, requested_zone, f);
        let components = DtfComponentsLocals::new(schema, f);
        self.emit_dtf_read_optional(
            &reader,
            &components.weekday,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.era,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.year,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.month,
            DateTimeMonthWidth::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.day,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.day_period,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.hour,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.minute,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.second,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.fractional,
            (1..=3).map(|v| {
                let d = DateTimeFractionalDigits::new(v).expect("closed fractional digits");
                (d, u64::from(d.get()))
            }),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.zone_name,
            TimeZoneNameStyle::ALL
                .into_iter()
                .map(|v| (v, v.code() as u64)),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.date_style,
            DateTimeStyle::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        self.emit_dtf_read_optional(
            &reader,
            &components.time_style,
            DateTimeStyle::ALL.iter().map(|v| (*v, v.wire_code())),
            f,
        );
        let mask = schema.reserve_i64_local(f);
        reader.read_u64(mask, schema, f);
        let availability = GcI64DomainLocal::new(
            schema,
            DateTimeFormatAvailability::from_available_kinds([]),
            f,
        );
        availability.set_checked_mask(mask, f);
        schema.release_i64_local(mask, f);
        let plan = self.emit_dtf_read_plan(&reader, f);
        reader.finish(schema, f);
        let calendar = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("", f)?, f);
        for kind in DateTimeCalendar::ALL {
            emit_domain_is(&locale.calendar, *kind, f);
            self.open_frame(ControlFrameKind::If, f);
            calendar.replace(self.emit_interned_string_reference(kind.as_str(), f)?, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let cycle = GcI32DomainLocal::new(schema, None::<DateTimeHourCycle>, f);
        let hour12 = GcI32DomainLocal::new(schema, None::<bool>, f);
        emit_domain_is(&components.hour, None, f);
        f.instruction(&Instruction::I32Eqz);
        emit_domain_is(&components.time_style, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        for kind in DateTimeHourCycle::ALL {
            emit_domain_is(&locale.hour_cycle, *kind, f);
            self.open_frame(ControlFrameKind::If, f);
            cycle.set_constant(Some(*kind), f);
            hour12.set_constant(
                Some(matches!(
                    kind,
                    DateTimeHourCycle::H11 | DateTimeHourCycle::H12
                )),
                f,
            );
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlDateTimeFormatObject>().construct(
                (
                    GcOperand::reference(&header.0, schema),
                    GcOperand::reference(&locale.locale, schema),
                    GcOperand::reference(&calendar, schema),
                    GcOperand::reference(&locale.numbering, schema),
                    GcOperand::reference(&zone.identifier, schema),
                    GcOperand::i64_local(zone.fixed_seconds),
                    zone.kind.operand(),
                    cycle.operand(),
                    components.weekday.operand(),
                    components.era.operand(),
                    components.year.operand(),
                    components.month.operand(),
                    components.day.operand(),
                    components.day_period.operand(),
                    components.hour.operand(),
                    components.minute.operand(),
                    components.second.operand(),
                    components.fractional.operand(),
                    components.zone_name.operand(),
                    components.date_style.operand(),
                    components.time_style.operand(),
                    hour12.operand(),
                    availability.operand(),
                    GcOperand::reference(&plan, schema),
                    GcOperand::null(schema),
                ),
                f,
            ),
            f,
        );
        hour12.clear(schema, f);
        cycle.clear(schema, f);
        calendar.clear(f);
        plan.clear(f);
        availability.clear(schema, f);
        components.clear(schema, f);
        zone.clear(schema, f);
        locale.clear(schema, f);
        header.0.clear(f);
        Ok(record)
    }
}
