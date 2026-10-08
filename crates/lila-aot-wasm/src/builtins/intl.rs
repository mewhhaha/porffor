//! Intl.Locale observations and completed GC records; native tag authority.
use super::super::*;
use super::intl_number::*;
use crate::functions::{NonArrayRealmIntrinsicSlot, OrdinaryDefaultPrototype};
use crate::gc_types::*;
use provider::CanonicalLocaleComponents;
mod calendars;
mod collations;
mod construction_lifecycle;
mod extension_options;
mod hour_cycles;
mod language_options;
mod likely_subtags;
mod locale_info_array;
mod locale_information_list;
mod numbering_systems;
mod provider;
mod text_info;
mod time_zones;
mod week_info;
/// A completed, deduplicated List of native-canonical String values. Only the
/// CanonicalizeLocaleList producer below can mint it.
pub(in crate::builtins) struct CanonicalLocaleListLocals {
    values: crate::gc_types::GcLocal<crate::gc_types::ValueArray>,
}
impl CanonicalLocaleListLocals {
    pub(in crate::builtins) fn values(
        &self,
    ) -> &crate::gc_types::GcLocal<crate::gc_types::ValueArray> {
        &self.values
    }
    pub(in crate::builtins) fn clear(self, function: &mut Function) {
        self.values.clear(function);
    }
}

#[derive(Clone, Copy)]
enum LocaleStringSlot {
    Tag,
    Language,
    Script,
    Region,
    BaseName,
}

impl FunctionBuilder<'_> {
    fn emit_intl_locale_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<IntlLocaleObject>, EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("Locale method lacks callable entry"))?
                .this_value(),
            function,
        );
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlLocaleObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_LOCALE_PROTOTYPE_METHOD_CALLED_ON_INCOMPATIBLE_RECEIVER,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<IntlLocaleObject>(schema, function),
            function,
        );
        receiver.clear(function);
        Ok(record)
    }
    fn emit_intl_locale_tag(
        &self,
        record: &GcLocal<IntlLocaleObject>,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let schema = self.runtime_schema();
        schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IntlLocaleObject>()
                .field(IntlLocaleObjectSchema::TAG)
                .read(record, schema, function)
                .reference(),
            function,
        )
    }
    pub(crate) fn emit_intl_locale_argument_to_string(
        &mut self,
        input: &ValueLocals,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let text = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        input.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlLocaleObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            input.cast_reference::<IntlLocaleObject>(schema, function),
            function,
        );
        let tag = self.emit_intl_locale_tag(&record, function);
        text.replace(tag.load(schema, function).nullable(), function);
        tag.clear(function);
        record.clear(function);
        function.instruction(&Instruction::Else);
        emit_tag_is(input, WasmRuntimeValueTag::String, function);
        self.emit_is_heap_object_like_tag_i32(input.tag(), function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(message, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let converted = self.emit_intl_number_to_string(input, function)?;
        text.replace(converted.load(schema, function).nullable(), function);
        converted.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let result = schema.reserve_gc_local(function).initialize(
            text.load(schema, function).require_non_null(function),
            function,
        );
        text.clear(function);
        Ok(result)
    }
    pub(crate) fn emit_intl_locale_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("Locale constructor lacks callable entry"))?
                .new_target(),
            function,
        );
        emit_tag_is(&target, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_LOCALE_CONSTRUCTOR_REQUIRES_NEW,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Reserve the result before observing the tag or options.
        let reserved = self.emit_reserve_intl_locale_object(&target, function)?;
        target.clear(function);
        let argument = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        let input = self.emit_intl_locale_argument_to_string(
            &argument,
            RuntimeErrorMessage::INTL_LOCALE_TAG_MUST_BE_A_STRING_OR_AN_OBJECT,
            function,
        )?;
        argument.clear(function);
        let options = self.emit_intl_locale_coerce_options(function)?;
        let tag = self.emit_intl_provider_locale_transform::<lila_intl::CanonicalizeLocale>(
            &input,
            RuntimeErrorMessage::INVALID_LANGUAGE_TAG,
            function,
        )?;
        input.clear(function);
        let components = self.emit_intl_locale_components(tag, function)?;
        let rebuilt = self.emit_intl_locale_language_options(&options, &components, function)?;
        components.clear(function);
        let tag = self.emit_intl_locale_extension_options(&options, rebuilt, function)?;
        options.clear(function);
        let canonical = self.emit_intl_provider_locale_transform::<lila_intl::CanonicalizeLocale>(
            &tag,
            RuntimeErrorMessage::INVALID_LANGUAGE_TAG,
            function,
        )?;
        tag.clear(function);
        let components = self.emit_intl_locale_components(canonical, function)?;
        let initialized = self.emit_initialize_intl_locale_object(reserved, &components, function);
        components.clear(function);
        self.emit_publish_intl_locale_object(initialized, function);
        Ok(())
    }
    fn emit_intl_locale_optional_string(
        &self,
        text: &GcLocal<StringValue, Nullable>,
        out: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        out.set_undefined(function);
        text.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let value = schema.reserve_gc_local(function).initialize(
            text.load(schema, function).require_non_null(function),
            function,
        );
        out.set_reference(&value, schema, function);
        value.clear(function);
        function.instruction(&Instruction::End);
    }
    pub(super) fn emit_intl_locale_language_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_string_getter(LocaleStringSlot::Language, f)
    }
    pub(super) fn emit_intl_locale_script_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_string_getter(LocaleStringSlot::Script, f)
    }
    pub(super) fn emit_intl_locale_region_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_string_getter(LocaleStringSlot::Region, f)
    }
    pub(super) fn emit_intl_locale_base_name_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_string_getter(LocaleStringSlot::BaseName, f)
    }
    pub(super) fn emit_intl_locale_to_string_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_string_getter(LocaleStringSlot::Tag, f)
    }
    fn emit_intl_locale_string_getter(
        &mut self,
        slot: LocaleStringSlot,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let output = schema.reserve_value_local(function);
        output.set_undefined(function);
        match slot {
            LocaleStringSlot::Tag | LocaleStringSlot::Language | LocaleStringSlot::BaseName => {
                let text = schema.reserve_gc_local(function).initialize(
                    match slot {
                        LocaleStringSlot::Tag => schema
                            .struct_type::<IntlLocaleObject>()
                            .field(IntlLocaleObjectSchema::TAG)
                            .read(&record, schema, function)
                            .reference(),
                        LocaleStringSlot::Language => schema
                            .struct_type::<IntlLocaleObject>()
                            .field(IntlLocaleObjectSchema::LANGUAGE)
                            .read(&record, schema, function)
                            .reference(),
                        LocaleStringSlot::BaseName => schema
                            .struct_type::<IntlLocaleObject>()
                            .field(IntlLocaleObjectSchema::BASE_NAME)
                            .read(&record, schema, function)
                            .reference(),
                        LocaleStringSlot::Script | LocaleStringSlot::Region => unreachable!(),
                    },
                    function,
                );
                output.set_reference(&text, schema, function);
                text.clear(function);
            }
            LocaleStringSlot::Script | LocaleStringSlot::Region => {
                let text = schema.reserve_gc_local(function).initialize(
                    match slot {
                        LocaleStringSlot::Script => schema
                            .struct_type::<IntlLocaleObject>()
                            .field(IntlLocaleObjectSchema::SCRIPT)
                            .read(&record, schema, function)
                            .reference(),
                        LocaleStringSlot::Region => schema
                            .struct_type::<IntlLocaleObject>()
                            .field(IntlLocaleObjectSchema::REGION)
                            .read(&record, schema, function)
                            .reference(),
                        LocaleStringSlot::Tag
                        | LocaleStringSlot::Language
                        | LocaleStringSlot::BaseName => unreachable!(),
                    },
                    function,
                );
                self.emit_intl_locale_optional_string(&text, &output, function);
                text.clear(function);
            }
        }
        self.completion().initialize(function);
        self.completion().value().copy_from(&output, function);
        output.clear(function);
        record.clear(function);
        Ok(())
    }
    pub(crate) fn emit_intl_get_canonical_locales(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        let locales = self.emit_intl_canonical_locale_list(&argument, function)?;
        let array = self.emit_array_from_argument_list(locales.values(), function)?;
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&array, schema, function);
        array.clear(function);
        locales.clear(function);
        argument.clear(function);
        Ok(())
    }

    pub(in crate::builtins) fn emit_intl_canonical_locale_list(
        &mut self,
        argument: &crate::gc_types::ValueLocals,
        function: &mut Function,
    ) -> Result<CanonicalLocaleListLocals, EmitError> {
        use crate::builtins::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
        use crate::emit::AccessorThrowRouting;
        use crate::gc_types::*;
        use crate::operations::PropertyKeyLocals;
        let schema = self.runtime_schema();
        let single = schema.reserve_i32_local(function);
        let count = schema.reserve_i64_local(function);
        let source = schema.reserve_value_local(function);
        source.set_undefined(function);
        let candidate = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        let values = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .fixed(core::iter::empty(), function),
            function,
        );
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        function.instruction(&Instruction::I32Eq);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlLocaleObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Or);
        single.store(function);
        single.load(function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        count.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_object_locals(argument, &pending, function)?;
        self.emit_intl_number_adopt_completion(&pending, function);
        source.copy_from(pending.value(), function);
        self.emit_intl_number_get_option(&source, "length", &candidate, function)?;
        self.emit_to_length_i64_from_value_locals(&candidate, count, &pending, function)?;
        self.emit_intl_number_adopt_completion(&pending, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let index = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        let present = schema.reserve_i32_local(function);
        let duplicate = schema.reserve_i32_local(function);
        let length = schema.reserve_i32_local(function);
        let inner = schema.reserve_i32_local(function);
        let key_number = schema.reserve_i64_local(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        single.load(function);
        self.open_frame(ControlFrameKind::If, function);
        candidate.copy_from(argument, function);
        function.instruction(&Instruction::I32Const(1));
        present.store(function);
        function.instruction(&Instruction::Else);
        index.load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        key_number.store(function);
        let key_text = schema.reserve_gc_local(function).initialize(
            self.emit_number_to_string_payload(key_number, function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &key_text, function);
        self.emit_object_has_property_i32(&source, &key, present, function)?;
        present.load(function);
        self.open_frame(ControlFrameKind::If, function);
        pending.initialize(function);
        self.emit_object_read_with_throw_routing(
            &source,
            &source,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        self.emit_intl_number_adopt_completion(&pending, function);
        candidate.copy_from(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        key.clear(function);
        key_text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        present.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let canonical_root = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        candidate.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlLocaleObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let locale = schema.reserve_gc_local(function).initialize(
            candidate.cast_reference::<IntlLocaleObject>(schema, function),
            function,
        );
        canonical_root.replace(
            schema
                .struct_type::<IntlLocaleObject>()
                .field(IntlLocaleObjectSchema::TAG)
                .read(&locale, schema, function)
                .reference()
                .nullable(),
            function,
        );
        locale.clear(function);
        function.instruction(&Instruction::Else);
        candidate.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_is_heap_object_like_tag_i32(candidate.tag(), function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_GETCANONICALLOCALES_LOCALE_MUST_BE_A_STRING_OR_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let text = self.emit_intl_number_to_string(&candidate, function)?;
        let request = IntlByteArrayBuilder::with_operation(
            lila_intl::IntlHostOp::CanonicalizeLocale,
            schema,
            function,
        );
        request.append_remaining_utf8(&text, schema, function);
        let request = request.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&request, function)?;
        reply.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_LANGUAGE_TAG, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let response = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        let reader = IntlByteArrayReader::new(&response, schema, function);
        let result = reader.consume_remaining_utf8(schema, function);
        reader.finish(schema, function);
        canonical_root.replace(result.load(schema, function).nullable(), function);
        result.clear(function);
        response.clear(function);
        reply.clear(function);
        request.clear(function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let canonical = schema.reserve_gc_local(function).initialize(
            canonical_root
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        canonical_root.clear(function);
        function.instruction(&Instruction::I32Const(0));
        duplicate.store(function);
        function.instruction(&Instruction::I32Const(0));
        inner.store(function);
        schema
            .array_type::<ValueArray>()
            .length(&values, schema, function);
        length.store(function);
        let searched = self.open_frame(ControlFrameKind::Block, function);
        let search = self.open_frame(ControlFrameKind::Loop, function);
        inner.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(searched, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .read(&values, inner, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &candidate, schema, function);
        stored.clear(function);
        let existing = schema.reserve_gc_local(function).initialize(
            candidate.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_string_payload_equality_i32(&canonical, &existing, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(1));
        duplicate.store(function);
        self.emit_branch_to_target(searched, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        existing.clear(function);
        inner.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        inner.store(function);
        self.emit_branch_to_target(search, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        duplicate.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        candidate.set_reference(&canonical, schema, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&candidate, function),
            function,
        );
        length.load(function);
        function.instruction(&Instruction::I32Const(i32::MAX));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        length.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        inner.store(function);
        let grown = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().filled(
                GcOperand::reference(&stored, schema),
                inner,
                function,
            ),
            function,
        );
        function.instruction(&Instruction::I32Const(0));
        inner.store(function);
        let copied = self.open_frame(ControlFrameKind::Block, function);
        let copy = self.open_frame(ControlFrameKind::Loop, function);
        inner.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(copied, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let old = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .read(&values, inner, schema, function)
                .reference(),
            function,
        );
        schema.array_type::<ValueArray>().write(
            &grown,
            inner,
            GcOperand::reference(&old, schema),
            schema,
            function,
        );
        old.clear(function);
        inner.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        inner.store(function);
        self.emit_branch_to_target(copy, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        values.replace(grown.load(schema, function), function);
        grown.clear(function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        canonical.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for local in [inner, length, duplicate, present] {
            schema.release_i32_local(local, function);
        }
        schema.release_i64_local(key_number, function);
        schema.release_i64_local(index, function);
        pending.clear(function);
        candidate.clear(function);
        source.clear(function);
        schema.release_i64_local(count, function);
        schema.release_i32_local(single, function);
        Ok(CanonicalLocaleListLocals { values })
    }
}
