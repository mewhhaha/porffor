use super::options::CompletedDurationOptionsLocals;
use super::*;
use crate::functions::OrdinaryDefaultPrototype;
struct ReservedDurationObject(GcLocal<OrdinaryObject>);
struct CompletedDurationInitialization {
    locale: GcLocal<StringValue>,
    numbering: GcLocal<StringValue>,
    options: CompletedDurationOptionsLocals,
}
impl FunctionBuilder<'_> {
    fn emit_reserve_duration_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedDurationObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(f);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DurationFormat constructor lacks callable entry")
                })?
                .new_target(),
            f,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DU_CONSTRUCT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlDurationFormat,
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
        Ok(ReservedDurationObject(header))
    }
    fn emit_complete_duration_initialization(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<CompletedDurationInitialization, EmitError> {
        let schema = self.runtime_schema();
        let requested = self.emit_intl_canonical_locale_list(locales, f)?;
        self.emit_duration_options_object(options, f)?;
        let matcher = GcI32DomainLocal::new(schema, LocaleMatcher::BestFit, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::LocaleMatcher,
            LocaleMatcher::ALL.iter().map(|v| (v.name(), *v)),
            LocaleMatcher::BestFit,
            &matcher,
            f,
        )?;
        let numbering_option = schema.reserve_value_local(f);
        self.emit_intl_number_string_value(options, "numberingSystem", &numbering_option, f)?;
        emit_tag_is(&numbering_option, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let string = schema
            .reserve_gc_local(f)
            .initialize(numbering_option.cast_reference::<StringValue>(schema, f), f);
        let valid = schema.reserve_i32_local(f);
        self.emit_intl_is_unicode_type_i32(&string, valid, f);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_NUMBERINGSYSTEM_OPTION, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(valid, f);
        string.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let response = self.emit_duration_provider_call(
            DurationProviderRequest::Resolve {
                locales: &requested,
                matcher: &matcher,
                numbering: &numbering_option,
            },
            f,
        )?;
        let reader = response.reader(schema, f);
        let locale = reader.read_utf8(schema, f);
        let numbering = reader.read_utf8(schema, f);
        let two_digit_word = schema.reserve_i64_local(f);
        reader.read_u64(two_digit_word, schema, f);
        two_digit_word.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let two_digit_hours = schema.reserve_i32_local(f);
        two_digit_word.load(f);
        f.instruction(&Instruction::I32WrapI64);
        two_digit_hours.store(f);
        reader.finish(schema, f);
        response.clear(f);
        let completed = self.emit_complete_duration_options(options, two_digit_hours, f)?;
        schema.release_i32_local(two_digit_hours, f);
        schema.release_i64_local(two_digit_word, f);
        numbering_option.clear(f);
        matcher.clear(schema, f);
        requested.clear(f);
        Ok(CompletedDurationInitialization {
            locale,
            numbering,
            options: completed,
        })
    }
    fn emit_publish_duration_record(
        &self,
        header: ReservedDurationObject,
        completed: CompletedDurationInitialization,
        f: &mut Function,
    ) -> GcLocal<IntlDurationFormatObject> {
        let schema = self.runtime_schema();
        let units = &completed.options.units;
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlDurationFormatObject>().construct(
                (
                    GcOperand::reference(&header.0, schema),
                    GcOperand::reference(&completed.locale, schema),
                    GcOperand::reference(&completed.numbering, schema),
                    completed.options.style.operand(),
                    completed.options.fractional_digits.operand(),
                    GcOperand::reference(&units[0], schema),
                    GcOperand::reference(&units[1], schema),
                    GcOperand::reference(&units[2], schema),
                    GcOperand::reference(&units[3], schema),
                    GcOperand::reference(&units[4], schema),
                    GcOperand::reference(&units[5], schema),
                    GcOperand::reference(&units[6], schema),
                    GcOperand::reference(&units[7], schema),
                    GcOperand::reference(&units[8], schema),
                    GcOperand::reference(&units[9], schema),
                ),
                f,
            ),
            f,
        );
        completed.options.clear(schema, f);
        completed.numbering.clear(f);
        completed.locale.clear(f);
        header.0.clear(f);
        record
    }
    pub(super) fn emit_initialize_duration_format_record(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDurationFormatObject>, EmitError> {
        // Temporal.Duration.toLocaleString creates the intrinsic service without a public lookup.
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let prototype = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IntlDurationFormatPrototype,
            &prototype,
            f,
        );
        let header = ReservedDurationObject(schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        ));
        prototype.clear(f);
        realm.clear(f);
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_duration_initialization(&locales, &options, f)?;
        let record = self.emit_publish_duration_record(header, completed, f);
        options.clear(f);
        locales.clear(f);
        Ok(record)
    }
    pub(crate) fn emit_intl_durationformat_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        // Observable NewTarget.prototype precedes locales/options observations.
        let header = self.emit_reserve_duration_object(f)?;
        let schema = self.runtime_schema();
        let locales = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &locales, f);
        self.emit_builtin_arg_to_value(1, &options, f);
        let completed = self.emit_complete_duration_initialization(&locales, &options, f)?;
        let record = self.emit_publish_duration_record(header, completed, f);
        self.completion().initialize(f);
        self.completion().value().set_reference(&record, schema, f);
        record.clear(f);
        options.clear(f);
        locales.clear(f);
        Ok(())
    }
}
