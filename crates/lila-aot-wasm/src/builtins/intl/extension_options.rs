//! Ordered Unicode keyword replacement on native-validated canonical tags.
use super::language_options::CoercedIntlLocaleOptions;
use super::*;
#[derive(Clone, Copy)]
pub(super) enum LocaleExtensionOption {
    Calendar,
    Collation,
    FirstDayOfWeek,
    HourCycle,
    CaseFirst,
    Numeric,
    NumberingSystem,
}

impl LocaleExtensionOption {
    const fn error_message(self) -> RuntimeErrorMessage {
        match self {
            Self::Calendar => RuntimeErrorMessage::INVALID_INTL_LOCALE_CALENDAR_OPTION,
            Self::Collation => RuntimeErrorMessage::INVALID_INTL_LOCALE_COLLATION_OPTION,
            Self::FirstDayOfWeek => RuntimeErrorMessage::INVALID_INTL_LOCALE_FIRSTDAYOFWEEK_OPTION,
            Self::HourCycle => RuntimeErrorMessage::INVALID_INTL_LOCALE_HOURCYCLE_OPTION,
            Self::CaseFirst => RuntimeErrorMessage::INVALID_INTL_LOCALE_CASEFIRST_OPTION,
            Self::Numeric => RuntimeErrorMessage::INVALID_INTL_LOCALE_NUMERIC_OPTION,
            Self::NumberingSystem => {
                RuntimeErrorMessage::INVALID_INTL_LOCALE_NUMBERINGSYSTEM_OPTION
            }
        }
    }

    const ORDERED: [Self; 7] = [
        Self::Calendar,
        Self::Collation,
        Self::FirstDayOfWeek,
        Self::HourCycle,
        Self::CaseFirst,
        Self::Numeric,
        Self::NumberingSystem,
    ];

    const fn property(self) -> &'static str {
        match self {
            Self::Calendar => "calendar",
            Self::Collation => "collation",
            Self::FirstDayOfWeek => "firstDayOfWeek",
            Self::HourCycle => "hourCycle",
            Self::CaseFirst => "caseFirst",
            Self::Numeric => "numeric",
            Self::NumberingSystem => "numberingSystem",
        }
    }

    pub(super) const fn key(self) -> &'static str {
        match self {
            Self::Calendar => "ca",
            Self::Collation => "co",
            Self::FirstDayOfWeek => "fw",
            Self::HourCycle => "hc",
            Self::CaseFirst => "kf",
            Self::Numeric => "kn",
            Self::NumberingSystem => "nu",
        }
    }
}

struct LocaleKeywordSpan {
    units: GcLocal<CodeUnitArray>,
    length: I32Local,
    unicode_start: I32Local,
    unicode_end: I32Local,
    insertion: I32Local,
    key_start: I32Local,
    key_end: I32Local,
}
impl LocaleKeywordSpan {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        for v in [
            self.key_end,
            self.key_start,
            self.insertion,
            self.unicode_end,
            self.unicode_start,
            self.length,
        ] {
            schema.release_i32_local(v, f);
        }
        self.units.clear(f);
    }
}
impl FunctionBuilder<'_> {
    fn emit_intl_locale_keyword_span(
        &self,
        tag: &GcLocal<StringValue>,
        option: LocaleExtensionOption,
        function: &mut Function,
    ) -> LocaleKeywordSpan {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(tag, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let unicode_start = schema.reserve_i32_local(function);
        let unicode_end = schema.reserve_i32_local(function);
        let insertion = schema.reserve_i32_local(function);
        let key_start = schema.reserve_i32_local(function);
        let key_end = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let end = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let second = schema.reserve_i32_local(function);
        let in_unicode = schema.reserve_i32_local(function);
        let private = schema.reserve_i32_local(function);
        let selected = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        for local in [
            unicode_start,
            key_start,
            key_end,
            index,
            in_unicode,
            private,
            selected,
        ] {
            set_i32(local, 0, function);
        }
        for local in [unicode_end, insertion] {
            length.load(function);
            local.store(function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        private.load(function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_locale_token_end(&units, length, index, end, function);
        end.load(function);
        index.load(function);
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        in_unicode.load(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        unicode_end.store(function);
        function.instruction(&Instruction::End);
        set_i32(in_unicode, 0, function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(b'u' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        unicode_start.store(function);
        set_i32(in_unicode, 1, function);
        function.instruction(&Instruction::End);
        unit.load(function);
        function.instruction(&Instruction::I32Const(b'x' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        insertion.store(function);
        set_i32(private, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        end.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        unicode_start.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        unicode_start.load(function);
        function.instruction(&Instruction::I32Const(3));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        unicode_end.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_locale_token_end(&units, unicode_end, index, end, function);
        end.load(function);
        index.load(function);
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(2));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        selected.load(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        key_end.store(function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        second.store(function);
        let key = option.key().as_bytes();
        unit.load(function);
        function.instruction(&Instruction::I32Const(i32::from(key[0])));
        function.instruction(&Instruction::I32Eq);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, second, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(i32::from(key[1])));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        key_start.store(function);
        unicode_end.load(function);
        key_end.store(function);
        set_i32(selected, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        end.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [selected, private, in_unicode, second, unit, end, index] {
            schema.release_i32_local(local, function);
        }
        LocaleKeywordSpan {
            units,
            length,
            unicode_start,
            unicode_end,
            insertion,
            key_start,
            key_end,
        }
    }
    pub(super) fn emit_intl_locale_extension_value(
        &self,
        tag: &GcLocal<StringValue>,
        option: LocaleExtensionOption,
        function: &mut Function,
    ) -> GcLocal<StringValue, Nullable> {
        let schema = self.runtime_schema();
        let span = self.emit_intl_locale_keyword_span(tag, option, function);
        let output = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        let start = schema.reserve_i32_local(function);
        span.key_start.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        span.key_start.load(function);
        function.instruction(&Instruction::I32Const(4));
        function.instruction(&Instruction::I32Add);
        start.store(function);
        start.load(function);
        span.key_end.load(function);
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        span.key_end.load(function);
        start.store(function);
        function.instruction(&Instruction::End);
        let selected = self.emit_intl_locale_slice(tag, start, span.key_end, function);
        output.replace(selected.load(schema, function).nullable(), function);
        selected.clear(function);
        function.instruction(&Instruction::End);
        schema.release_i32_local(start, function);
        span.clear(schema, function);
        output
    }
    fn emit_intl_locale_replace_keyword(
        &mut self,
        tag: &GcLocal<StringValue>,
        option: LocaleExtensionOption,
        value: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let span = self.emit_intl_locale_keyword_span(tag, option, function);
        let zero = schema.reserve_i32_local(function);
        let start = schema.reserve_i32_local(function);
        let end = schema.reserve_i32_local(function);
        set_i32(zero, 0, function);
        span.key_start.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        span.unicode_start.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        span.insertion.load(function);
        start.store(function);
        function.instruction(&Instruction::Else);
        span.unicode_end.load(function);
        start.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        start.load(function);
        end.store(function);
        function.instruction(&Instruction::Else);
        span.key_start.load(function);
        start.store(function);
        span.key_end.load(function);
        end.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let prefix = self.emit_intl_locale_slice(tag, zero, start, function);
        let suffix = self.emit_intl_locale_slice(tag, end, span.length, function);
        span.unicode_start.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let u = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("-u", function)?,
            function,
        );
        prefix.replace(self.emit_concat_gc_strings(&prefix, &u, function), function);
        u.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let separator = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("-", function)?,
            function,
        );
        let key = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(option.key(), function)?,
            function,
        );
        prefix.replace(
            self.emit_concat_gc_strings(&prefix, &separator, function),
            function,
        );
        prefix.replace(
            self.emit_concat_gc_strings(&prefix, &key, function),
            function,
        );
        prefix.replace(
            self.emit_concat_gc_strings(&prefix, &separator, function),
            function,
        );
        prefix.replace(
            self.emit_concat_gc_strings(&prefix, value, function),
            function,
        );
        tag.replace(
            self.emit_concat_gc_strings(&prefix, &suffix, function),
            function,
        );
        key.clear(function);
        separator.clear(function);
        suffix.clear(function);
        prefix.clear(function);
        for local in [end, start, zero] {
            schema.release_i32_local(local, function);
        }
        span.clear(schema, function);
        Ok(())
    }
    fn emit_intl_locale_validate_extension_option(
        &mut self,
        option: LocaleExtensionOption,
        text: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        if matches!(option, LocaleExtensionOption::FirstDayOfWeek) {
            for (number, day) in [
                (0, "sun"),
                (1, "mon"),
                (2, "tue"),
                (3, "wed"),
                (4, "thu"),
                (5, "fri"),
                (6, "sat"),
                (7, "sun"),
            ] {
                let expected = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(&number.to_string(), function)?,
                    function,
                );
                self.emit_string_payload_equality_i32(text, &expected, function);
                self.open_frame(ControlFrameKind::If, function);
                text.replace(
                    self.emit_interned_string_reference(day, function)?,
                    function,
                );
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                expected.clear(function);
            }
        }
        let allowed: &[&str] = match option {
            LocaleExtensionOption::HourCycle => &["h11", "h12", "h23", "h24"],
            LocaleExtensionOption::CaseFirst => &["upper", "lower", "false"],
            LocaleExtensionOption::Calendar
            | LocaleExtensionOption::Collation
            | LocaleExtensionOption::FirstDayOfWeek
            | LocaleExtensionOption::Numeric
            | LocaleExtensionOption::NumberingSystem => &[],
        };
        if !allowed.is_empty() {
            function.instruction(&Instruction::I32Const(0));
            for candidate in allowed {
                let expected = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(candidate, function)?,
                    function,
                );
                self.emit_string_payload_equality_i32(text, &expected, function);
                function.instruction(&Instruction::I32Or);
                expected.clear(function);
            }
            return self.emit_intl_locale_option_guard(option.error_message(), function);
        }
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
        let end = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let folded = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        set_i32(index, 0, function);
        length.load(function);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32GtU);
        self.emit_intl_locale_option_guard(option.error_message(), function)?;
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
        self.emit_intl_locale_range(unit, 3, 8, function);
        self.emit_intl_locale_option_guard(option.error_message(), function)?;
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
        self.emit_intl_locale_option_guard(option.error_message(), function)?;
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
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32LtU);
        self.emit_intl_locale_option_guard(option.error_message(), function)?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [folded, unit, end, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        Ok(())
    }
    pub(super) fn emit_intl_locale_extension_options(
        &mut self,
        options: &CoercedIntlLocaleOptions,
        tag: GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        for option in LocaleExtensionOption::ORDERED {
            self.emit_intl_number_get_option(
                options.receiver(),
                option.property(),
                &value,
                function,
            )?;
            emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            let text = if matches!(option, LocaleExtensionOption::Numeric) {
                let text = schema
                    .reserve_gc_local::<StringValue, Nullable>(function)
                    .initialize_null(schema, function);
                self.compile_truthy_tagged_i32(&value, function)?;
                self.open_frame(ControlFrameKind::If, function);
                text.replace(
                    self.emit_interned_string_reference("true", function)?
                        .nullable(),
                    function,
                );
                function.instruction(&Instruction::Else);
                text.replace(
                    self.emit_interned_string_reference("false", function)?
                        .nullable(),
                    function,
                );
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                let result = schema.reserve_gc_local(function).initialize(
                    text.load(schema, function).require_non_null(function),
                    function,
                );
                text.clear(function);
                result
            } else {
                let text = self.emit_intl_number_to_string(&value, function)?;
                self.emit_intl_locale_validate_extension_option(option, &text, function)?;
                text
            };
            self.emit_intl_locale_replace_keyword(&tag, option, &text, function)?;
            text.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        value.clear(function);
        Ok(tag)
    }
    fn emit_intl_locale_extension_getter(
        &mut self,
        option: LocaleExtensionOption,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let text = self.emit_intl_locale_extension_value(&tag, option, function);
        let output = schema.reserve_value_local(function);
        self.emit_intl_locale_optional_string(&text, &output, function);
        if matches!(option, LocaleExtensionOption::Numeric) {
            text.load(schema, function).is_null(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            let selected = schema.reserve_gc_local(function).initialize(
                text.load(schema, function).require_non_null(function),
                function,
            );
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference("true", function)?,
                function,
            );
            self.emit_string_payload_equality_i32(&selected, &expected, function);
            let length = schema.reserve_i32_local(function);
            let units = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(&selected, schema, function)
                    .reference(),
                function,
            );
            schema
                .array_type::<CodeUnitArray>()
                .length(&units, schema, function);
            length.store(function);
            units.clear(function);
            length.load(function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32Or);
            length.store(function);
            output.set_boolean(length, function);
            schema.release_i32_local(length, function);
            expected.clear(function);
            selected.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.completion().initialize(function);
        self.completion().value().copy_from(&output, function);
        output.clear(function);
        text.clear(function);
        tag.clear(function);
        record.clear(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_locale_variants_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let components = self.emit_intl_locale_components(tag, function)?;
        let schema = self.runtime_schema();
        let output = schema.reserve_value_local(function);
        self.emit_intl_locale_optional_string(&components.variants, &output, function);
        self.completion().initialize(function);
        self.completion().value().copy_from(&output, function);
        output.clear(function);
        components.clear(function);
        record.clear(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_locale_calendar_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Calendar, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_collation_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Collation, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_first_day_of_week_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::FirstDayOfWeek, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_hour_cycle_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::HourCycle, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_case_first_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::CaseFirst, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_numeric_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Numeric, f)
    }
    pub(in crate::builtins) fn emit_intl_locale_numbering_system_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::NumberingSystem, f)
    }
}
