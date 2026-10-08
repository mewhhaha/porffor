use super::*;
use crate::functions::OrdinaryDefaultPrototype;
struct ReservedListFormatObject(GcLocal<OrdinaryObject>);
struct CompletedListFormatInitialization {
    locale: GcLocal<StringValue>,
    kind: GcI32DomainLocal<ListType>,
    style: GcI32DomainLocal<ListStyle>,
}
impl FunctionBuilder<'_> {
    fn emit_reserve_list_format_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedListFormatObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(f);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("ListFormat constructor lacks callable entry")
                })?
                .new_target(),
            f,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(LF_CONSTRUCT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlListFormat,
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
        Ok(ReservedListFormatObject(header))
    }
    fn emit_complete_list_format_initialization(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<CompletedListFormatInitialization, EmitError> {
        let schema = self.runtime_schema();
        let requested = self.emit_intl_canonical_locale_list(locales, f)?;
        // Retain this constructor's existing strict options-object admission.
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
            RuntimeErrorMessage::INTL_LISTFORMAT_OPTIONS_MUST_BE_AN_OBJECT,
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
        let response = self.emit_list_provider_call(
            ListProviderRequest::Resolve {
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
        let kind = GcI32DomainLocal::new(schema, ListType::Conjunction, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Type,
            ListType::ALL.iter().map(|v| (v.name(), *v)),
            ListType::Conjunction,
            &kind,
            f,
        )?;
        let style = GcI32DomainLocal::new(schema, ListStyle::Long, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Style,
            ListStyle::ALL.iter().map(|v| (v.name(), *v)),
            ListStyle::Long,
            &style,
            f,
        )?;
        Ok(CompletedListFormatInitialization {
            locale,
            kind,
            style,
        })
    }
    fn emit_publish_list_format_object(
        &self,
        reserved: ReservedListFormatObject,
        completed: CompletedListFormatInitialization,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlListFormatObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(&completed.locale, schema),
                    completed.kind.operand(),
                    completed.style.operand(),
                ),
                f,
            ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        completed.style.clear(schema, f);
        completed.kind.clear(schema, f);
        completed.locale.clear(f);
        reserved.0.clear(f);
    }
    pub(crate) fn emit_intl_list_format_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reserved = self.emit_reserve_list_format_object(f)?;
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_list_format_initialization(&locales, &options, f)?;
        self.emit_publish_list_format_object(reserved, completed, f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
}
