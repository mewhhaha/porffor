use super::*;
/// Only the ordered locales/options/usage/ResolveOptions producer mints this.
pub(super) struct ObservedCollatorResolutionInputsLocals {
    locales: CanonicalLocaleListLocals,
    matcher: GcI32DomainLocal<LocaleMatcher>,
    usage: GcI32DomainLocal<CollatorUsage>,
    collation_present: I32Local,
    collation: GcLocal<StringValue>,
    numeric_present: I32Local,
    numeric: I32Local,
    case_first: GcI32DomainLocal<Option<CollatorCaseFirst>>,
}
impl ObservedCollatorResolutionInputsLocals {
    pub(super) fn locales(&self) -> &CanonicalLocaleListLocals {
        &self.locales
    }
    pub(super) fn matcher(&self) -> &GcI32DomainLocal<LocaleMatcher> {
        &self.matcher
    }
    pub(super) fn usage(&self) -> &GcI32DomainLocal<CollatorUsage> {
        &self.usage
    }
    pub(super) fn collation_present(&self) -> I32Local {
        self.collation_present
    }
    pub(super) fn collation(&self) -> &GcLocal<StringValue> {
        &self.collation
    }
    pub(super) fn numeric_present(&self) -> I32Local {
        self.numeric_present
    }
    pub(super) fn numeric(&self) -> I32Local {
        self.numeric
    }
    pub(super) fn case_first(&self) -> &GcI32DomainLocal<Option<CollatorCaseFirst>> {
        &self.case_first
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, function: &mut Function) {
        self.case_first.clear(schema, function);
        schema.release_i32_local(self.numeric, function);
        schema.release_i32_local(self.numeric_present, function);
        self.collation.clear(function);
        schema.release_i32_local(self.collation_present, function);
        self.usage.clear(schema, function);
        self.matcher.clear(schema, function);
        self.locales.clear(function);
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_observed_collator_resolution_inputs(
        &mut self,
        locales: &ValueLocals,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<ObservedCollatorResolutionInputsLocals, EmitError> {
        let schema = self.runtime_schema();
        let locales = self.emit_intl_canonical_locale_list(locales, function)?;
        self.emit_intl_number_options_object(options, function)?;
        let usage = GcI32DomainLocal::new(schema, CollatorUsage::Sort, function);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Usage,
            CollatorUsage::ALL.iter().map(|v| (v.name(), *v)),
            CollatorUsage::Sort,
            &usage,
            function,
        )?;
        let matcher = GcI32DomainLocal::new(schema, LocaleMatcher::BestFit, function);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::LocaleMatcher,
            LocaleMatcher::ALL.iter().map(|v| (v.name(), *v)),
            LocaleMatcher::BestFit,
            &matcher,
            function,
        )?;
        let collation_present = schema.reserve_i32_local(function);
        set_i32(collation_present, 0, function);
        let collation = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_string_value(options, "collation", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let valid = schema.reserve_i32_local(function);
        self.emit_intl_is_unicode_type_i32(&text, valid, function);
        valid.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_COLLATION_OPTION, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        collation.replace(text.load(schema, function), function);
        set_i32(collation_present, 1, function);
        schema.release_i32_local(valid, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let numeric_present = schema.reserve_i32_local(function);
        set_i32(numeric_present, 0, function);
        let numeric = schema.reserve_i32_local(function);
        set_i32(numeric, 0, function);
        self.emit_intl_number_get_option(options, "numeric", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_tagged_i32(&value, function)?;
        numeric.store(function);
        set_i32(numeric_present, 1, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let case_first = GcI32DomainLocal::new(schema, None, function);
        self.emit_intl_number_string_value(options, "caseFirst", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let recognized = schema.reserve_i32_local(function);
        set_i32(recognized, 0, function);
        for variant in CollatorCaseFirst::ALL {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(variant.name(), function)?,
                function,
            );
            self.emit_string_payload_equality_i32(&text, &expected, function);
            self.open_frame(ControlFrameKind::If, function);
            case_first.set_constant(Some(*variant), function);
            set_i32(recognized, 1, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            expected.clear(function);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(IntlErrorOption::CaseFirst.error_message(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        Ok(ObservedCollatorResolutionInputsLocals {
            locales,
            matcher,
            usage,
            collation_present,
            collation,
            numeric_present,
            numeric,
            case_first,
        })
    }
    pub(super) fn emit_collator_boolean_option(
        &mut self,
        options: &ValueLocals,
        property: &str,
        destination: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_get_option(options, property, &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_tagged_i32(&value, function)?;
        destination.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        Ok(())
    }
}
