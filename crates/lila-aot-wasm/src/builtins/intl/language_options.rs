//! Each core option Get, ToString and validation completes before the next Get.
use super::*;
#[must_use]
pub(super) struct CoercedIntlLocaleOptions(ValueLocals);
impl CoercedIntlLocaleOptions {
    pub(super) fn receiver(&self) -> &ValueLocals {
        &self.0
    }
    pub(super) fn clear(self, f: &mut Function) {
        self.0.clear(f);
    }
}
#[derive(Clone, Copy)]
enum LanguageOption {
    Language,
    Script,
    Region,
    Variants,
}
impl LanguageOption {
    fn property(self) -> &'static str {
        match self {
            Self::Language => "language",
            Self::Script => "script",
            Self::Region => "region",
            Self::Variants => "variants",
        }
    }
    fn error(self) -> RuntimeErrorMessage {
        match self {
            Self::Language => RuntimeErrorMessage::INVALID_INTL_LOCALE_LANGUAGE_OPTION,
            Self::Script => RuntimeErrorMessage::INVALID_INTL_LOCALE_SCRIPT_OPTION,
            Self::Region => RuntimeErrorMessage::INVALID_INTL_LOCALE_REGION_OPTION,
            Self::Variants => RuntimeErrorMessage::INVALID_INTL_LOCALE_VARIANTS_OPTION,
        }
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_intl_locale_coerce_options(
        &mut self,
        function: &mut Function,
    ) -> Result<CoercedIntlLocaleOptions, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(1, &value, function);
        emit_tag_is(&value, WasmRuntimeValueTag::Null, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_LOCALE_OPTIONS_MUST_NOT_BE_NULL,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_intl_number_options_object(&value, function)?;
        Ok(CoercedIntlLocaleOptions(value))
    }
    pub(super) fn emit_intl_locale_range(
        &self,
        value: I32Local,
        min: i32,
        max: i32,
        function: &mut Function,
    ) {
        value.load(function);
        function.instruction(&Instruction::I32Const(min));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(max - min));
        function.instruction(&Instruction::I32LeU);
    }
    pub(super) fn emit_intl_locale_option_guard(
        &mut self,
        error: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(error, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_intl_locale_validate_core(
        &mut self,
        option: LanguageOption,
        text: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(text, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let folded = schema.reserve_i32_local(function);
        let alpha = schema.reserve_i32_local(function);
        let digit = schema.reserve_i32_local(function);
        let end = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        set_i32(index, 0, function);
        if matches!(option, LanguageOption::Variants) {
            length.load(function);
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32GtU);
            self.emit_intl_locale_option_guard(option.error(), function)?;
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            length.load(function);
            function.instruction(&Instruction::I32GeU);
            function.instruction(&Instruction::BrIf(1));
            self.emit_intl_locale_token_end(&units, length, index, end, function);
            end.load(function);
            index.load(function);
            function.instruction(&Instruction::I32Sub);
            unit.store(function);
            self.emit_intl_locale_range(unit, 5, 8, function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(4));
            function.instruction(&Instruction::I32Eq);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, function)
                .store(unit, function);
            self.emit_intl_locale_range(unit, b'0' as i32, b'9' as i32, function);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            self.emit_intl_locale_option_guard(option.error(), function)?;
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            end.load(function);
            function.instruction(&Instruction::I32GeU);
            function.instruction(&Instruction::BrIf(1));
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, function)
                .store(unit, function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(32));
            function.instruction(&Instruction::I32Or);
            folded.store(function);
            self.emit_intl_locale_range(folded, b'a' as i32, b'z' as i32, function);
            self.emit_intl_locale_range(unit, b'0' as i32, b'9' as i32, function);
            function.instruction(&Instruction::I32Or);
            self.emit_intl_locale_option_guard(option.error(), function)?;
            index.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            end.load(function);
            length.load(function);
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::BrIf(1));
            end.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            // A trailing separator is never an empty accepted variant.
            index.load(function);
            length.load(function);
            function.instruction(&Instruction::I32LtU);
            self.emit_intl_locale_option_guard(option.error(), function)?;
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            let prefix = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference("und-", function)?,
                function,
            );
            let candidate = schema.reserve_gc_local(function).initialize(
                self.emit_concat_gc_strings(&prefix, text, function),
                function,
            );
            // The native parse checks duplicate variants before applying aliases.
            let checked = self
                .emit_intl_provider_locale_transform::<lila_intl::CanonicalizeLocale>(
                    &candidate,
                    option.error(),
                    function,
                )?;
            checked.clear(function);
            candidate.clear(function);
            prefix.clear(function);
        } else {
            match option {
                LanguageOption::Language => {
                    self.emit_intl_locale_range(length, 2, 3, function);
                    self.emit_intl_locale_range(length, 5, 8, function);
                    function.instruction(&Instruction::I32Or);
                }
                LanguageOption::Script => self.emit_intl_locale_range(length, 4, 4, function),
                LanguageOption::Region => self.emit_intl_locale_range(length, 2, 3, function),
                LanguageOption::Variants => unreachable!(),
            }
            self.emit_intl_locale_option_guard(option.error(), function)?;
            set_i32(alpha, 1, function);
            set_i32(digit, 1, function);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            length.load(function);
            function.instruction(&Instruction::I32GeU);
            function.instruction(&Instruction::BrIf(1));
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, function)
                .store(unit, function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(32));
            function.instruction(&Instruction::I32Or);
            folded.store(function);
            alpha.load(function);
            self.emit_intl_locale_range(folded, b'a' as i32, b'z' as i32, function);
            function.instruction(&Instruction::I32And);
            alpha.store(function);
            digit.load(function);
            self.emit_intl_locale_range(unit, b'0' as i32, b'9' as i32, function);
            function.instruction(&Instruction::I32And);
            digit.store(function);
            index.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            alpha.load(function);
            if matches!(option, LanguageOption::Region) {
                self.emit_intl_locale_range(length, 2, 2, function);
                function.instruction(&Instruction::I32And);
                digit.load(function);
                self.emit_intl_locale_range(length, 3, 3, function);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
            }
            self.emit_intl_locale_option_guard(option.error(), function)?;
        }
        for local in [end, digit, alpha, folded, unit, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        Ok(())
    }
    pub(super) fn emit_intl_locale_language_options(
        &mut self,
        options: &CoercedIntlLocaleOptions,
        components: &CanonicalLocaleComponents,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        // Working overrides never mutate the completed canonical component view.
        let language = schema
            .reserve_gc_local(function)
            .initialize(components.language.load(schema, function), function);
        let script = schema
            .reserve_gc_local(function)
            .initialize(components.script.load(schema, function), function);
        let region = schema
            .reserve_gc_local(function)
            .initialize(components.region.load(schema, function), function);
        let variants = schema
            .reserve_gc_local(function)
            .initialize(components.variants.load(schema, function), function);
        let value = schema.reserve_value_local(function);
        for (option, destination) in [
            (LanguageOption::Language, None),
            (LanguageOption::Script, Some(&script)),
            (LanguageOption::Region, Some(&region)),
            (LanguageOption::Variants, Some(&variants)),
        ] {
            self.emit_intl_number_get_option(
                options.receiver(),
                option.property(),
                &value,
                function,
            )?;
            emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            let text = self.emit_intl_number_to_string(&value, function)?;
            self.emit_intl_locale_validate_core(option, &text, function)?;
            if let Some(destination) = destination {
                destination.replace(text.load(schema, function).nullable(), function);
            } else {
                language.replace(text.load(schema, function), function);
            }
            text.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        let rebuilt = schema
            .reserve_gc_local(function)
            .initialize(language.load(schema, function), function);
        let separator = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("-", function)?,
            function,
        );
        for component in [&script, &region, &variants] {
            component.load(schema, function).is_null(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            let selected = schema.reserve_gc_local(function).initialize(
                component.load(schema, function).require_non_null(function),
                function,
            );
            let joined = schema.reserve_gc_local(function).initialize(
                self.emit_concat_gc_strings(&rebuilt, &separator, function),
                function,
            );
            rebuilt.replace(
                self.emit_concat_gc_strings(&joined, &selected, function),
                function,
            );
            joined.clear(function);
            selected.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        rebuilt.replace(
            self.emit_concat_gc_strings(&rebuilt, &components.suffix, function),
            function,
        );
        separator.clear(function);
        value.clear(function);
        variants.clear(function);
        region.clear(function);
        script.clear(function);
        language.clear(function);
        Ok(rebuilt)
    }
}
