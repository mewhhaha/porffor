use super::*;
use crate::functions::OrdinaryDefaultPrototype;
struct ReservedPluralRulesObject(GcLocal<OrdinaryObject>);
/// Only completed locale resolution and all digit options can publish the record.
struct CompletedPluralRulesInitialization {
    locale: GcLocal<StringValue>,
    data_locale: GcLocal<StringValue>,
    kind: GcI32DomainLocal<PluralType>,
    categories: GcI32DomainLocal<PluralCategorySet>,
    notation: GcI32DomainLocal<NotationOption>,
    compact_display: GcI32DomainLocal<Option<CompactDisplay>>,
    rounding: GcLocal<IntlNumberRounding>,
}
impl FunctionBuilder<'_> {
    fn emit_reserve_plural_rules_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedPluralRulesObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(f);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("PluralRules constructor lacks callable entry")
                })?
                .new_target(),
            f,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(PR_CONSTRUCT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlPluralRules,
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
        Ok(ReservedPluralRulesObject(header))
    }
    fn emit_complete_plural_rules_initialization(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<CompletedPluralRulesInitialization, EmitError> {
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
        let response = self.emit_plural_provider_call(
            PluralProviderRequest::Resolve {
                locales: &requested,
                matcher: &matcher,
            },
            f,
        )?;
        let reader = response.reader(schema, f);
        let locale = reader.read_utf8(schema, f);
        let data_locale = reader.read_utf8(schema, f);
        let cardinal = GcI32DomainLocal::new(schema, plural_other_categories(), f);
        let ordinal = GcI32DomainLocal::new(schema, plural_other_categories(), f);
        let word = schema.reserve_i64_local(f);
        reader.read_u64(word, schema, f);
        cardinal.set_checked_mask(word, f);
        reader.read_u64(word, schema, f);
        ordinal.set_checked_mask(word, f);
        reader.finish(schema, f);
        schema.release_i64_local(word, f);
        response.clear(f);
        matcher.clear(schema, f);
        requested.clear(f);
        let kind = GcI32DomainLocal::new(schema, PluralType::Cardinal, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Type,
            PluralType::ALL.iter().map(|v| (v.name(), *v)),
            PluralType::Cardinal,
            &kind,
            f,
        )?;
        let notation = GcI32DomainLocal::new(schema, NotationOption::Standard, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Notation,
            NotationOption::ALL.iter().map(|v| (v.name(), *v)),
            NotationOption::Standard,
            &notation,
            f,
        )?;
        let compact = GcI32DomainLocal::new(schema, CompactDisplay::Short, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::CompactDisplay,
            CompactDisplay::ALL.iter().map(|v| (v.name(), *v)),
            CompactDisplay::Short,
            &compact,
            f,
        )?;
        let rounding = self.emit_intl_number_digit_options(
            options,
            &notation,
            IntlDigitDefaults::PluralRules,
            f,
        )?;
        let compact_display = GcI32DomainLocal::new(schema, None, f);
        emit_domain_is(&notation, NotationOption::Compact, f);
        self.open_frame(ControlFrameKind::If, f);
        for variant in CompactDisplay::ALL {
            emit_domain_is(&compact, *variant, f);
            self.open_frame(ControlFrameKind::If, f);
            compact_display.set_constant(Some(*variant), f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        compact.clear(schema, f);
        let categories = GcI32DomainLocal::new(schema, plural_other_categories(), f);
        emit_domain_is(&kind, PluralType::Ordinal, f);
        self.open_frame(ControlFrameKind::If, f);
        categories.copy_from(&ordinal, f);
        f.instruction(&Instruction::Else);
        categories.copy_from(&cardinal, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        ordinal.clear(schema, f);
        cardinal.clear(schema, f);
        Ok(CompletedPluralRulesInitialization {
            locale,
            data_locale,
            kind,
            categories,
            notation,
            compact_display,
            rounding,
        })
    }
    fn emit_publish_plural_rules_object(
        &self,
        reserved: ReservedPluralRulesObject,
        completed: CompletedPluralRulesInitialization,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlPluralRulesObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(&completed.locale, schema),
                    GcOperand::reference(&completed.data_locale, schema),
                    completed.kind.operand(),
                    completed.categories.operand(),
                    completed.notation.operand(),
                    completed.compact_display.operand(),
                    GcOperand::reference(&completed.rounding, schema),
                ),
                f,
            ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        completed.rounding.clear(f);
        completed.compact_display.clear(schema, f);
        completed.notation.clear(schema, f);
        completed.categories.clear(schema, f);
        completed.kind.clear(schema, f);
        completed.data_locale.clear(f);
        completed.locale.clear(f);
        reserved.0.clear(f);
    }
    pub(crate) fn emit_intl_plural_rules_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reserved = self.emit_reserve_plural_rules_object(f)?;
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_plural_rules_initialization(&locales, &options, f)?;
        self.emit_publish_plural_rules_object(reserved, completed, f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
}
