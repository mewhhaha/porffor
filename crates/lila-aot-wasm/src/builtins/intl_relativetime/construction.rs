use super::*;
use crate::functions::OrdinaryDefaultPrototype;
struct ReservedRelativeTimeObject(GcLocal<OrdinaryObject>);
struct CompletedRelativeTimeInitialization {
    locale: GcLocal<StringValue>,
    formatting: GcLocal<StringValue>,
    numbering: GcLocal<StringValue>,
    style: GcI32DomainLocal<RelativeStyle>,
    numeric: GcI32DomainLocal<RelativeNumeric>,
}
impl FunctionBuilder<'_> {
    fn emit_reserve_relative_time_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedRelativeTimeObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(f);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("RelativeTimeFormat constructor lacks callable entry")
                })?
                .new_target(),
            f,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RT_CONSTRUCT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlRelativeTimeFormat,
            &pending,
            f,
        )?;
        self.emit_intl_number_adopt_completion(&pending, f);
        let header = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(pending.value()), f)?,
            f,
        );
        pending.clear(f);
        target.clear(f);
        Ok(ReservedRelativeTimeObject(header))
    }
    fn emit_relative_numbering_option(
        &mut self,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let valid = schema.reserve_i32_local(f);
        let output = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("", f)?, f);
        self.emit_intl_number_string_value(options, "numberingSystem", &value, f)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let text = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(schema, f), f);
        self.emit_intl_is_unicode_type_i32(&text, valid, f);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_NUMBERINGSYSTEM_OPTION, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output.replace(text.load(schema, f), f);
        text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(valid, f);
        value.clear(f);
        Ok(output)
    }
    fn emit_complete_relative_time_initialization(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<CompletedRelativeTimeInitialization, EmitError> {
        let schema = self.runtime_schema();
        let requested = self.emit_intl_canonical_locale_list(locales, f)?;
        self.emit_intl_number_options_object(options, f)?;
        let matcher = GcI32DomainLocal::new(schema, LocaleMatcher::BestFit, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::LocaleMatcher,
            LocaleMatcher::ALL.iter().map(|v| (v.name(), *v)),
            LocaleMatcher::BestFit,
            &matcher,
            f,
        )?;
        let requested_numbering = self.emit_relative_numbering_option(options, f)?;
        let response = self.emit_relative_provider_call(
            RelativeProviderRequest::Resolve {
                locales: &requested,
                matcher: &matcher,
                numbering: &requested_numbering,
            },
            f,
        )?;
        let reader = response.reader(schema, f);
        let locale = reader.read_utf8(schema, f);
        let formatting = reader.read_utf8(schema, f);
        let numbering = reader.read_utf8(schema, f);
        reader.finish(schema, f);
        response.clear(f);
        requested_numbering.clear(f);
        matcher.clear(schema, f);
        requested.clear(f);
        let style = GcI32DomainLocal::new(schema, RelativeStyle::Long, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Style,
            RelativeStyle::ALL.iter().map(|v| (v.name(), *v)),
            RelativeStyle::Long,
            &style,
            f,
        )?;
        let numeric = GcI32DomainLocal::new(schema, RelativeNumeric::Always, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Numeric,
            RelativeNumeric::ALL.iter().map(|v| (v.name(), *v)),
            RelativeNumeric::Always,
            &numeric,
            f,
        )?;
        Ok(CompletedRelativeTimeInitialization {
            locale,
            formatting,
            numbering,
            style,
            numeric,
        })
    }
    fn emit_publish_relative_time_object(
        &self,
        reserved: ReservedRelativeTimeObject,
        completed: CompletedRelativeTimeInitialization,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlRelativeTimeFormatObject>()
                .construct(
                    (
                        GcOperand::reference(&reserved.0, schema),
                        GcOperand::reference(&completed.locale, schema),
                        GcOperand::reference(&completed.formatting, schema),
                        GcOperand::reference(&completed.numbering, schema),
                        completed.style.operand(),
                        completed.numeric.operand(),
                    ),
                    f,
                ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        completed.numeric.clear(schema, f);
        completed.style.clear(schema, f);
        completed.numbering.clear(f);
        completed.formatting.clear(f);
        completed.locale.clear(f);
        reserved.0.clear(f);
    }
    pub(crate) fn emit_intl_relative_time_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reserved = self.emit_reserve_relative_time_object(f)?;
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_relative_time_initialization(&locales, &options, f)?;
        self.emit_publish_relative_time_object(reserved, completed, f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
}
