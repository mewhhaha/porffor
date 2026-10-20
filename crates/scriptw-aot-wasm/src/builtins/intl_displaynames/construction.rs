use super::*;
use crate::functions::OrdinaryDefaultPrototype;
struct ReservedDisplayNamesObject(GcLocal<OrdinaryObject>);
struct CompletedDisplayNamesInitialization {
    locale: GcLocal<StringValue>,
    kind: GcI32DomainLocal<DisplayNamesType>,
    style: GcI32DomainLocal<DisplayNamesStyle>,
    fallback: GcI32DomainLocal<DisplayNamesFallback>,
    language: GcI32DomainLocal<Option<DisplayNamesLanguageDisplay>>,
}
impl FunctionBuilder<'_> {
    fn emit_reserve_display_names_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedDisplayNamesObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(f);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DisplayNames constructor lacks callable entry")
                })?
                .new_target(),
            f,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DN_CONSTRUCT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlDisplayNames,
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
        Ok(ReservedDisplayNamesObject(header))
    }
    fn emit_display_names_required_type(
        &mut self,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<GcI32DomainLocal<DisplayNamesType>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_intl_number_get_option(options, "type", &value, f)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DN_REQUIRED_TYPE_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let text = self.emit_intl_number_to_string(&value, f)?;
        let kind = GcI32DomainLocal::new(schema, DisplayNamesType::Language, f);
        let recognized = schema.reserve_i32_local(f);
        set_i32(recognized, 0, f);
        for candidate in DisplayNamesType::ALL {
            let expected = schema
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(candidate.name(), f)?, f);
            self.emit_string_payload_equality_i32(&text, &expected, f);
            self.open_frame(ControlFrameKind::If, f);
            kind.set_constant(*candidate, f);
            set_i32(recognized, 1, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            expected.clear(f);
        }
        recognized.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_TYPE_OPTION, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(recognized, f);
        text.clear(f);
        value.clear(f);
        Ok(kind)
    }
    fn emit_complete_display_names_initialization(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<CompletedDisplayNamesInitialization, EmitError> {
        let schema = self.runtime_schema();
        let requested = self.emit_intl_canonical_locale_list(locales, f)?;
        // Preserve the constructor's current strict options-object admission.
        emit_tag_is(options, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        let empty = schema
            .reserve_gc_local(f)
            .initialize(self.emit_alloc_plain_object_with_prototype(None, f)?, f);
        options.set_reference(&empty, schema, f);
        empty.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(options.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_DISPLAYNAMES_OPTIONS_MUST_BE_AN_OBJECT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let matcher = GcI32DomainLocal::new(schema, LocaleMatcher::BestFit, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::LocaleMatcher,
            LocaleMatcher::ALL.iter().map(|v| (v.name(), *v)),
            LocaleMatcher::BestFit,
            &matcher,
            f,
        )?;
        let response = self.emit_display_names_provider_call(
            DisplayNamesProviderRequest::Resolve {
                locales: &requested,
                matcher: &matcher,
            },
            f,
        )?;
        let reader = response.reader(schema, f);
        let locale = reader.read_utf8(schema, f);
        reader.finish(schema, f);
        response.clear(f);
        matcher.clear(schema, f);
        requested.clear(f);
        let style = GcI32DomainLocal::new(schema, DisplayNamesStyle::Long, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Style,
            DisplayNamesStyle::ALL.iter().map(|v| (v.name(), *v)),
            DisplayNamesStyle::Long,
            &style,
            f,
        )?;
        let kind = self.emit_display_names_required_type(options, f)?;
        let fallback = GcI32DomainLocal::new(schema, DisplayNamesFallback::Code, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Fallback,
            DisplayNamesFallback::ALL.iter().map(|v| (v.name(), *v)),
            DisplayNamesFallback::Code,
            &fallback,
            f,
        )?;
        // Every type observes and validates languageDisplay; only Language stores it.
        let observed = GcI32DomainLocal::new(schema, DisplayNamesLanguageDisplay::Dialect, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::LanguageDisplay,
            DisplayNamesLanguageDisplay::ALL
                .iter()
                .map(|v| (v.name(), *v)),
            DisplayNamesLanguageDisplay::Dialect,
            &observed,
            f,
        )?;
        let language = GcI32DomainLocal::new(schema, None, f);
        emit_domain_is(&kind, DisplayNamesType::Language, f);
        self.open_frame(ControlFrameKind::If, f);
        for candidate in DisplayNamesLanguageDisplay::ALL {
            emit_domain_is(&observed, *candidate, f);
            self.open_frame(ControlFrameKind::If, f);
            language.set_constant(Some(*candidate), f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        observed.clear(schema, f);
        Ok(CompletedDisplayNamesInitialization {
            locale,
            kind,
            style,
            fallback,
            language,
        })
    }
    fn emit_publish_display_names_object(
        &self,
        reserved: ReservedDisplayNamesObject,
        completed: CompletedDisplayNamesInitialization,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlDisplayNamesObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(&completed.locale, schema),
                    completed.kind.operand(),
                    completed.style.operand(),
                    completed.fallback.operand(),
                    completed.language.operand(),
                ),
                f,
            ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        completed.language.clear(schema, f);
        completed.fallback.clear(schema, f);
        completed.style.clear(schema, f);
        completed.kind.clear(schema, f);
        completed.locale.clear(f);
        reserved.0.clear(f);
    }
    pub(crate) fn emit_intl_display_names_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reserved = self.emit_reserve_display_names_object(f)?;
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_display_names_initialization(&locales, &options, f)?;
        self.emit_publish_display_names_object(reserved, completed, f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
}
