//! DateTimeFormat's ordered observations, completed GC recipe and native provider boundary.
use super::super::*;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::functions::{
    ArgumentListConstruction, NonArrayRealmIntrinsicSlot, OrdinaryDefaultPrototype,
};
use crate::gc_types::*;
use lila_intl::{
    DateTimeCalendar, DateTimeDefaults, DateTimeFormatAvailability, DateTimeFormatMatcher,
    DateTimeFractionalDigits, DateTimeHourCycle, DateTimeHourCyclePreference,
    DateTimeLocaleMatcher, DateTimeMonthWidth, DateTimeNumericWidth, DateTimePartKind,
    DateTimeRangeSource, DateTimeRequired, DateTimeStyle, DateTimeTextWidth, DateTimeValueKind,
    FixedTimeZoneOffset, IntlHostOp, SystemTimeZoneKind, TimeZoneKind, TimeZoneNameStyle,
    DATE_TIME_WIRE_VERSION,
};
mod construction_lifecycle;
mod initialization;
mod provider_input;
mod provider_render;
mod provider_wire;
mod time_zone;
mod zoned_locale;
use initialization::DtfComponentsLocals;
pub(crate) use initialization::IntlDateTimeFormatPurpose;
use provider_input::CompletedDtfInput;
use provider_wire::{dtf_append_domain, dtf_append_optional, DtfProviderResponse};
use time_zone::ResolvedDtfTimeZone;

const INTL_DTF_EMPTY_TEMPORAL_FORMAT: RuntimeErrorMessage =
    RuntimeErrorMessage::THE_REQUESTED_FORMAT_HAS_NO_FIELDS_IN_COMMON_WITH_THIS_TEMPORAL_TYPE;
const INTL_DTF_CALENDAR_MISMATCH: RuntimeErrorMessage =
    RuntimeErrorMessage::TEMPORAL_CALENDAR_DOES_NOT_MATCH_THE_FORMATTER_CALENDAR;
const INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE: RuntimeErrorMessage =
    RuntimeErrorMessage::UNSUPPORTED_TIMEZONE_OPTION;
#[derive(Clone, Copy)]
pub(crate) enum DtfTemporalKind {
    PlainDate,
    PlainYearMonth,
    PlainMonthDay,
    PlainTime,
    PlainDateTime,
    Instant,
}
impl DtfTemporalKind {
    fn rejected_style(
        self,
    ) -> Option<(initialization::RejectedDateTimeStyle, RuntimeErrorMessage)> {
        use initialization::RejectedDateTimeStyle as S;
        match self {
 Self::PlainDate=>Some((S::Time,RuntimeErrorMessage::TEMPORAL_PLAINDATE_PROTOTYPE_TOLOCALESTRING_DOES_NOT_SUPPORT_THE_TIMESTYLE_OPTION)),
 Self::PlainYearMonth=>Some((S::Time,RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_TOLOCALESTRING_DOES_NOT_SUPPORT_THE_TIMESTYLE_OPTION)),
 Self::PlainMonthDay=>Some((S::Time,RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_TOLOCALESTRING_DOES_NOT_SUPPORT_THE_TIMESTYLE_OPTION)),
 Self::PlainTime=>Some((S::Date,RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_TOLOCALESTRING_DOES_NOT_SUPPORT_THE_DATESTYLE_OPTION)),
 Self::PlainDateTime|Self::Instant=>None }
    }
}
#[derive(Clone, Copy)]
enum DtfReceiverOperation {
    ResolvedOptions,
    FormatGetter,
    FormatToParts,
    FormatRange,
    FormatRangeToParts,
}
impl DtfReceiverOperation {
    fn error(self) -> RuntimeErrorMessage {
        match self {
 Self::ResolvedOptions=>RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_RESOLVEDOPTIONS_CALLED_ON_A_NON_INTL_DATETIMEFORMAT_OBJECT,
 Self::FormatGetter=>RuntimeErrorMessage::GET_INTL_DATETIMEFORMAT_PROTOTYPE_FORMAT_CALLED_ON_A_NON_INTL_DATETIMEFORMAT_OBJECT,
 Self::FormatToParts=>RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_FORMATTOPARTS_CALLED_ON_A_NON_INTL_DATETIMEFORMAT_OBJECT,
 Self::FormatRange=>RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_FORMATRANGE_CALLED_ON_A_NON_INTL_DATETIMEFORMAT_OBJECT,
 Self::FormatRangeToParts=>RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_FORMATRANGETOPARTS_CALLED_ON_A_NON_INTL_DATETIMEFORMAT_OBJECT }
    }
}
impl FunctionBuilder<'_> {
    fn emit_dtf_record_from_receiver(
        &mut self,
        operation: DtfReceiverOperation,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDateTimeFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DateTimeFormat receiver lacks callable entry")
                })?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlDateTimeFormatObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(operation.error(), f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(f).initialize(
            value.cast_reference::<IntlDateTimeFormatObject>(schema, f),
            f,
        );
        value.clear(f);
        Ok(record)
    }
    pub(crate) fn emit_intl_date_time_format_resolved_options(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let record =
            self.emit_dtf_record_from_receiver(DtfReceiverOperation::ResolvedOptions, f)?;
        let schema = self.runtime_schema();
        let dtf = schema.struct_type::<IntlDateTimeFormatObject>();
        let object = self.emit_intl_number_result_object(f)?;
        let value = schema.reserve_value_local(f);
        let text = schema.reserve_gc_local(f).initialize(
            dtf.field(IntlDateTimeFormatObjectSchema::LOCALE)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        value.set_reference(&text, schema, f);
        self.emit_intl_number_append_result_property(&object, "locale", &value, f)?;
        text.clear(f);
        let text = schema.reserve_gc_local(f).initialize(
            dtf.field(IntlDateTimeFormatObjectSchema::CALENDAR)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        value.set_reference(&text, schema, f);
        self.emit_intl_number_append_result_property(&object, "calendar", &value, f)?;
        text.clear(f);
        let text = schema.reserve_gc_local(f).initialize(
            dtf.field(IntlDateTimeFormatObjectSchema::NUMBERING_SYSTEM)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        value.set_reference(&text, schema, f);
        self.emit_intl_number_append_result_property(&object, "numberingSystem", &value, f)?;
        text.clear(f);
        let text = schema.reserve_gc_local(f).initialize(
            dtf.field(IntlDateTimeFormatObjectSchema::TIME_ZONE)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        value.set_reference(&text, schema, f);
        self.emit_intl_number_append_result_property(&object, "timeZone", &value, f)?;
        text.clear(f);
        let components = DtfComponentsLocals::new(schema, f);
        components.read_record(&record, schema, f);
        emit_domain_is(&components.hour, None, f);
        f.instruction(&Instruction::I32Eqz);
        emit_domain_is(&components.time_style, None, f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        let cycle = GcI32DomainLocal::new(schema, None::<DateTimeHourCycle>, f);
        dtf.field(IntlDateTimeFormatObjectSchema::HOUR_CYCLE)
            .read(&record, schema, f)
            .store_domain(&cycle, f);
        self.emit_dtf_resolved_option(
            &object,
            "hourCycle",
            &cycle,
            DateTimeHourCycle::ALL.iter().map(|v| (*v, v.as_str())),
            f,
        )?;
        let boolean = schema.reserve_i32_local(f);
        emit_domain_is(&cycle, Some(DateTimeHourCycle::H11), f);
        emit_domain_is(&cycle, Some(DateTimeHourCycle::H12), f);
        f.instruction(&Instruction::I32Or);
        boolean.store(f);
        value.set_boolean(boolean, f);
        self.emit_intl_number_append_result_property(&object, "hour12", &value, f)?;
        schema.release_i32_local(boolean, f);
        cycle.clear(schema, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&components.date_style, None, f);
        emit_domain_is(&components.time_style, None, f);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Weekday.property(),
            &components.weekday,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Era.property(),
            &components.era,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Year.property(),
            &components.year,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Month.property(),
            &components.month,
            DateTimeMonthWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Day.property(),
            &components.day,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::DayPeriod.property(),
            &components.day_period,
            DateTimeTextWidth::ALL.iter().map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Hour.property(),
            &components.hour,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Minute.property(),
            &components.minute,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::Second.property(),
            &components.second,
            DateTimeNumericWidth::ALL
                .iter()
                .map(|v| (*v, v.option_name())),
            f,
        )?;
        emit_domain_is(&components.fractional, None, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let bits = schema.reserve_i64_local(f);
        components.fractional.load(f);
        f.instruction(&Instruction::F64ConvertI32U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        value.set_number(bits, f);
        self.emit_intl_number_append_result_property(&object, "fractionalSecondDigits", &value, f)?;
        schema.release_i64_local(bits, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::TimeZoneName.property(),
            &components.zone_name,
            TimeZoneNameStyle::ALL
                .into_iter()
                .map(|v| (v, v.spelling())),
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::DateStyle.property(),
            &components.date_style,
            DateTimeStyle::ALL.iter().map(|v| (*v, v.option_name())),
            f,
        )?;
        self.emit_dtf_resolved_option(
            &object,
            IntlErrorOption::TimeStyle.property(),
            &components.time_style,
            DateTimeStyle::ALL.iter().map(|v| (*v, v.option_name())),
            f,
        )?;
        self.completion().initialize(f);
        self.completion().value().set_reference(&object, schema, f);
        components.clear(schema, f);
        value.clear(f);
        object.clear(f);
        record.clear(f);
        Ok(())
    }
    fn emit_dtf_resolved_option<V: GcI32Constant + Copy>(
        &mut self,
        object: &GcLocal<OrdinaryObject>,
        property: &str,
        selected: &GcI32DomainLocal<Option<V>>,
        variants: impl IntoIterator<Item = (V, &'static str)>,
        f: &mut Function,
    ) -> Result<(), EmitError>
    where
        Option<V>: GcI32Constant,
    {
        let schema = self.runtime_schema();
        emit_domain_is(selected, None, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let value = schema.reserve_value_local(f);
        for (variant, name) in variants {
            emit_domain_is(selected, Some(variant), f);
            self.open_frame(ControlFrameKind::If, f);
            let text = schema
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(name, f)?, f);
            value.set_reference(&text, schema, f);
            text.clear(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.emit_intl_number_append_result_property(object, property, &value, f)?;
        value.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(crate) fn emit_intl_date_time_format_format_getter(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_dtf_record_from_receiver(DtfReceiverOperation::FormatGetter, f)?;
        let schema = self.runtime_schema();
        let dtf = schema.struct_type::<IntlDateTimeFormatObject>();
        let bound = schema
            .reserve_gc_local::<FunctionObject, Nullable>(f)
            .initialize(
                dtf.field(IntlDateTimeFormatObjectSchema::BOUND_FORMAT)
                    .read(&record, schema, f)
                    .reference(),
                f,
            );
        bound.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatBoundFormat.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing DateTimeFormat bound format"))?;
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(f)
            .initialize(
                schema
                    .struct_type::<BuiltinClosureCapture>()
                    .publish(
                        BuiltinClosurePayload::IntlDateTimeFormat(&record),
                        schema,
                        f,
                    )
                    .nullable(),
                f,
            );
        let callable = schema.reserve_gc_local(f).initialize(
            self.emit_current_builtin_realm_closure_value(&meta, &capture, f)?,
            f,
        );
        dtf.field(IntlDateTimeFormatObjectSchema::BOUND_FORMAT)
            .write(
                &record,
                GcOperand::nullable_reference(&callable, schema),
                schema,
                f,
            );
        bound.replace(callable.load(schema, f).nullable(), f);
        callable.clear(f);
        capture.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let callable = schema
            .reserve_gc_local(f)
            .initialize(bound.load(schema, f).require_non_null(f), f);
        self.completion().initialize(f);
        self.completion()
            .value()
            .set_reference(&callable, schema, f);
        callable.clear(f);
        bound.clear(f);
        record.clear(f);
        Ok(())
    }
    pub(crate) fn emit_intl_dtf_temporal_to_locale_string(
        &mut self,
        kind: DtfTemporalKind,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("Temporal locale entry absent"))?
                .this_value(),
            f,
        );
        self.emit_dtf_require_temporal_receiver(kind, &value, f)?;
        self.emit_intl_dtf_format_value_with_new_formatter(kind, &value, f)?;
        value.clear(f);
        Ok(())
    }
    pub(crate) fn emit_intl_dtf_format_value_with_new_formatter(
        &mut self,
        kind: DtfTemporalKind,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_create_date_time_format(IntlDateTimeFormatPurpose::Temporal(kind), f)?;
        let schema = self.runtime_schema();
        let formatter = schema.reserve_value_local(f);
        formatter.copy_from(self.completion().value(), f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let getter = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing DateTimeFormat format getter"))?;
        self.emit_direct_js_call(&getter, Some(&formatter), &[], &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        let callable = schema.reserve_value_local(f);
        callable.copy_from(pending.value(), f);
        let this = schema.reserve_value_local(f);
        this.set_undefined(f);
        let args = self.emit_pre_evaluated_arg_vector(&[value], f);
        self.emit_function_or_proxy_call_with_argv(&callable, &this, &args, &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        args.clear(f);
        this.clear(f);
        callable.clear(f);
        pending.clear(f);
        formatter.clear(f);
        Ok(())
    }
}
pub(crate) fn intl_date_time_format_pool_strings() -> Vec<String> {
    let mut values = vec![
        "Intl.DateTimeFormat",
        "locale",
        "calendar",
        "numberingSystem",
        "timeZone",
        "hourCycle",
        "hour12",
        "localeMatcher",
        "formatMatcher",
        "type",
        "value",
        "source",
        "dateStyle",
        "timeStyle",
        "fractionalSecondDigits",
        "UTC",
        "iso8601",
        "",
        "+",
        "-",
        ":",
    ];
    for names in [
        DateTimeTextWidth::OPTIONS,
        DateTimeNumericWidth::OPTIONS,
        DateTimeMonthWidth::OPTIONS,
        DateTimeStyle::OPTIONS,
        DateTimeHourCycle::OPTIONS,
        DateTimeLocaleMatcher::OPTIONS,
        DateTimeFormatMatcher::OPTIONS,
    ] {
        values.extend(names.iter().map(|(name, _)| *name));
    }
    values.extend([
        "weekday",
        "era",
        "year",
        "month",
        "day",
        "dayPeriod",
        "hour",
        "minute",
        "second",
        "timeZoneName",
    ]);
    values.extend(DateTimeCalendar::ALL.iter().map(|v| v.as_str()));
    values.extend(TimeZoneNameStyle::ALL.iter().map(|v| v.spelling()));
    values.extend(DateTimePartKind::ALL.iter().map(|v| v.as_str()));
    values.extend(DateTimeRangeSource::ALL.iter().map(|v| v.as_str()));
    values.into_iter().map(str::to_owned).collect()
}
