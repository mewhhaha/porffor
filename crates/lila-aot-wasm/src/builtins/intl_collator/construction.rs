use super::*;
use crate::functions::{NonArrayRealmIntrinsicSlot, OrdinaryDefaultPrototype};
struct ReservedCollatorObject(GcLocal<OrdinaryObject>);
struct ResolvedCollatorInitializationLocals {
    locale: GcLocal<StringValue>,
    collation: GcLocal<StringValue>,
    usage: GcI32DomainLocal<CollatorUsage>,
    sensitivity: GcI32DomainLocal<CollatorSensitivity>,
    case_first: GcI32DomainLocal<CollatorCaseFirst>,
    collation_kind: GcI32DomainLocal<CollatorCollationKind>,
    numeric: I32Local,
    ignore_punctuation: I32Local,
}
/// Only completed provider resolution plus the final two option Gets can publish.
struct CompletedCollatorInitializationLocals(ResolvedCollatorInitializationLocals);
impl ResolvedCollatorInitializationLocals {
    fn clear(self, schema: &RuntimeSchema, function: &mut Function) {
        schema.release_i32_local(self.ignore_punctuation, function);
        schema.release_i32_local(self.numeric, function);
        self.collation_kind.clear(schema, function);
        self.case_first.clear(schema, function);
        self.sensitivity.clear(schema, function);
        self.usage.clear(schema, function);
        self.collation.clear(function);
        self.locale.clear(function);
    }
}
impl FunctionBuilder<'_> {
    fn emit_reserve_collator_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedCollatorObject, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Collator constructor lacks actual callable entry")
                })?
                .new_target(),
            function,
        );
        let prototype = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IntlCollatorPrototype,
            &prototype,
            function,
        );
        realm.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlCollator,
            &pending,
            function,
        )?;
        self.emit_intl_number_adopt_completion(&pending, function);
        prototype.copy_from(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        pending.clear(function);
        prototype.clear(function);
        target.clear(function);
        Ok(ReservedCollatorObject(object))
    }
    fn emit_collator_resolution_from_response(
        &mut self,
        input: &ObservedCollatorResolutionInputsLocals,
        response: &CollatorProviderResponse,
        function: &mut Function,
    ) -> Result<ResolvedCollatorInitializationLocals, EmitError> {
        let schema = self.runtime_schema();
        let reader = response.reader(schema, function);
        let locale = reader.read_utf8(schema, function);
        let collation_kind =
            GcI32DomainLocal::new(schema, CollatorCollationKind::Default, function);
        self.emit_collator_read_domain(
            &reader,
            &collation_kind,
            CollatorCollationKind::ALL
                .iter()
                .map(|v| (*v, v.wire_code())),
            function,
        );
        let collation = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        emit_domain_is(
            &collation_kind,
            CollatorCollationKind::UnicodeType,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        let text = reader.read_utf8(schema, function);
        let valid = schema.reserve_i32_local(function);
        self.emit_intl_is_unicode_type_i32(&text, valid, function);
        valid.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        collation.replace(text.load(schema, function), function);
        schema.release_i32_local(valid, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let numeric = schema.reserve_i32_local(function);
        self.emit_collator_read_boolean(&reader, numeric, function);
        let case_first = GcI32DomainLocal::new(schema, CollatorCaseFirst::False, function);
        self.emit_collator_read_domain(
            &reader,
            &case_first,
            CollatorCaseFirst::ALL.iter().map(|v| (*v, v.wire_code())),
            function,
        );
        let sensitivity = GcI32DomainLocal::new(schema, CollatorSensitivity::Variant, function);
        self.emit_collator_read_domain(
            &reader,
            &sensitivity,
            CollatorSensitivity::ALL.iter().map(|v| (*v, v.wire_code())),
            function,
        );
        let ignore_punctuation = schema.reserve_i32_local(function);
        self.emit_collator_read_boolean(&reader, ignore_punctuation, function);
        reader.finish(schema, function);
        let usage = GcI32DomainLocal::new(schema, CollatorUsage::Sort, function);
        usage.copy_from(input.usage(), function);
        Ok(ResolvedCollatorInitializationLocals {
            locale,
            collation,
            usage,
            sensitivity,
            case_first,
            collation_kind,
            numeric,
            ignore_punctuation,
        })
    }
    fn emit_complete_collator_options(
        &mut self,
        resolved: ResolvedCollatorInitializationLocals,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<CompletedCollatorInitializationLocals, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_string_value(options, "sensitivity", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let recognized = schema.reserve_i32_local(function);
        set_i32(recognized, 0, function);
        for variant in CollatorSensitivity::ALL {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(variant.name(), function)?,
                function,
            );
            self.emit_string_payload_equality_i32(&text, &expected, function);
            self.open_frame(ControlFrameKind::If, function);
            resolved.sensitivity.set_constant(*variant, function);
            set_i32(recognized, 1, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            expected.clear(function);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(IntlErrorOption::Sensitivity.error_message(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        self.emit_collator_boolean_option(
            options,
            "ignorePunctuation",
            resolved.ignore_punctuation,
            function,
        )?;
        Ok(CompletedCollatorInitializationLocals(resolved))
    }
    fn emit_publish_collator_object(
        &self,
        reserved: ReservedCollatorObject,
        completed: CompletedCollatorInitializationLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let config = completed.0;
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IntlCollatorObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(&config.locale, schema),
                    GcOperand::reference(&config.collation, schema),
                    config.usage.operand(),
                    config.sensitivity.operand(),
                    config.case_first.operand(),
                    config.collation_kind.operand(),
                    GcOperand::boolean_local(config.numeric),
                    GcOperand::boolean_local(config.ignore_punctuation),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&record, schema, function);
        record.clear(function);
        config.clear(schema, function);
        reserved.0.clear(function);
    }
    pub(crate) fn emit_intl_collator_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reserved = self.emit_reserve_collator_object(function)?;
        let locales = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &locales, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        let observed =
            self.emit_observed_collator_resolution_inputs(&locales, &options, function)?;
        let response = self
            .emit_collator_provider_call(CollatorProviderRequest::Resolve(&observed), function)?;
        let resolved =
            self.emit_collator_resolution_from_response(&observed, &response, function)?;
        let completed = self.emit_complete_collator_options(resolved, &options, function)?;
        self.emit_publish_collator_object(reserved, completed, function);
        response.clear(function);
        observed.clear(schema, function);
        options.clear(function);
        locales.clear(function);
        Ok(())
    }
}
