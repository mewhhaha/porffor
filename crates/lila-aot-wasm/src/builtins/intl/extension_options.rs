//! Ordered Unicode-extension options and views into immutable Locale tags.

use super::language_options::CoercedIntlLocaleOptions;
use super::*;

#[derive(Clone, Copy)]
enum LocaleExtensionOption {
    Calendar,
    Collation,
    FirstDayOfWeek,
    HourCycle,
    CaseFirst,
    Numeric,
    NumberingSystem,
}

impl LocaleExtensionOption {
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

    const fn key(self) -> &'static str {
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

impl<'a> FunctionBuilder<'a> {
    fn emit_intl_add_const(&self, local: u32, amount: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(amount));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(local));
    }

    pub(super) fn emit_intl_locale_extension_options(
        &mut self,
        options: &CoercedIntlLocaleOptions,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options = options.receiver();
        let key = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let value_tag = self.reserve_temp_local();
        let result = (|| {
            function.instruction(&Instruction::LocalGet(options.tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            for option in LocaleExtensionOption::ORDERED {
                self.emit_intl_set_const(key, self.strings.payload(option.property()), function);
                self.emit_object_read(
                    options.payload,
                    options.tag,
                    options.payload,
                    options.tag,
                    key,
                    value,
                    value_tag,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                match option {
                    LocaleExtensionOption::Numeric => {
                        self.compile_truthy_tagged_i32(value_tag, value, function)?;
                        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalSet(value));
                    }
                    LocaleExtensionOption::Calendar
                    | LocaleExtensionOption::Collation
                    | LocaleExtensionOption::FirstDayOfWeek
                    | LocaleExtensionOption::HourCycle
                    | LocaleExtensionOption::CaseFirst
                    | LocaleExtensionOption::NumberingSystem => {
                        self.emit_value_to_string_payload(value, value_tag, function)?;
                        function.instruction(&Instruction::LocalSet(value));
                        self.emit_return_current_completion_if_throw(function);
                        self.emit_intl_locale_validate_extension_option(option, value, function)?;
                    }
                }
                self.emit_intl_locale_replace_keyword(tag, option, value, function)?;
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            Ok(())
        })();
        for local in [value_tag, value, key] {
            self.release_temp_local(local);
        }
        result
    }

    fn emit_intl_locale_validate_extension_option(
        &mut self,
        option: LocaleExtensionOption,
        value: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected = self.reserve_temp_local();
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let folded = self.reserve_temp_local();
        let subtag_length = self.reserve_temp_local();
        let result = (|| {
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
                    self.emit_intl_set_const(
                        expected,
                        self.strings.payload(&number.to_string()),
                        function,
                    );
                    self.emit_string_payload_equality_i32(value, expected, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_intl_set_const(value, self.strings.payload(day), function);
                    function.instruction(&Instruction::End);
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
                    self.emit_intl_set_const(expected, self.strings.payload(candidate), function);
                    self.emit_string_payload_equality_i32(value, expected, function);
                    function.instruction(&Instruction::I32Or);
                }
                self.emit_intl_locale_extension_option_guard(option, function)?;
                return Ok(());
            }
            self.emit_unpack_string_payload(value, offset, length, function);
            self.emit_intl_set_const(index, 0, function);
            self.emit_intl_set_const(subtag_length, 0, function);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            self.emit_load_string_byte(offset, index, byte, function);
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(b'-' as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_locale_in_range(subtag_length, 3, 8, function);
            self.emit_intl_locale_extension_option_guard(option, function)?;
            self.emit_intl_set_const(subtag_length, 0, function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(folded));
            self.emit_intl_locale_in_range(folded, b'a' as i64, b'z' as i64, function);
            self.emit_intl_locale_in_range(byte, b'0' as i64, b'9' as i64, function);
            function.instruction(&Instruction::I32Or);
            self.emit_intl_locale_extension_option_guard(option, function)?;
            self.emit_intl_add_const(subtag_length, 1, function);
            function.instruction(&Instruction::End);
            self.emit_intl_add_const(index, 1, function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.emit_intl_locale_in_range(subtag_length, 3, 8, function);
            self.emit_intl_locale_extension_option_guard(option, function)?;
            Ok(())
        })();
        for local in [subtag_length, folded, byte, index, length, offset, expected] {
            self.release_temp_local(local);
        }
        result
    }

    fn emit_intl_locale_extension_option_guard(
        &mut self,
        option: LocaleExtensionOption,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            &format!("Invalid Intl.Locale {} option", option.property()),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    // The input is a validated tag. Bounds include each extension's leading
    // separator; private-use content is never interpreted as extension keys.
    fn emit_intl_locale_unicode_bounds(
        &mut self,
        tag: u32,
        unicode_start: u32,
        unicode_end: u32,
        insertion: u32,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let in_unicode = self.reserve_temp_local();
        let private = self.reserve_temp_local();
        self.emit_unpack_string_payload(tag, offset, length, function);
        for local in [unicode_start, in_unicode, private, index] {
            self.emit_intl_set_const(local, 0, function);
        }
        for local in [unicode_end, insertion] {
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(private));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_locale_token_end(offset, length, index, end, function);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(in_unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(unicode_end));
        function.instruction(&Instruction::End);
        self.emit_intl_set_const(in_unicode, 0, function);
        self.emit_load_string_byte(offset, index, byte, function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(unicode_start));
        self.emit_intl_set_const(in_unicode, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'x' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(insertion));
        self.emit_intl_set_const(private, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [private, in_unicode, byte, end, index, length, offset] {
            self.release_temp_local(local);
        }
    }

    fn emit_intl_locale_token_end(
        &mut self,
        offset: u32,
        limit: u32,
        start: u32,
        end: u32,
        function: &mut Function,
    ) {
        let byte = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::LocalSet(end));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset, end, byte, function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_add_const(end, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(byte);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_intl_locale_keyword_span(
        &mut self,
        tag: u32,
        option: LocaleExtensionOption,
        unicode_start: u32,
        unicode_end: u32,
        key_start: u32,
        key_end: u32,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let code = self.reserve_temp_local();
        let second = self.reserve_temp_local();
        let selected = self.reserve_temp_local();
        self.emit_unpack_string_payload(tag, offset, length, function);
        self.emit_intl_set_const(key_start, 0, function);
        self.emit_intl_set_const(key_end, 0, function);
        self.emit_intl_set_const(selected, 0, function);
        function.instruction(&Instruction::LocalGet(unicode_start));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unicode_start));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(unicode_end));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_locale_token_end(offset, unicode_end, index, end, function);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(selected));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(key_end));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(offset, index, code, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(second));
        self.emit_load_string_byte(offset, second, byte, function);
        function.instruction(&Instruction::LocalGet(code));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Or);
        let key = option.key().as_bytes();
        function.instruction(&Instruction::I64Const(
            ((key[0] as i64) << 8) | key[1] as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(key_start));
        function.instruction(&Instruction::LocalGet(unicode_end));
        function.instruction(&Instruction::LocalSet(key_end));
        self.emit_intl_set_const(selected, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [selected, second, code, byte, end, index, length, offset] {
            self.release_temp_local(local);
        }
    }

    fn emit_intl_locale_replace_keyword(
        &mut self,
        tag: u32,
        option: LocaleExtensionOption,
        value: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let unicode_start = self.reserve_temp_local();
        let unicode_end = self.reserve_temp_local();
        let insertion = self.reserve_temp_local();
        let start = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let scratch = self.reserve_temp_local();
        let size = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let position = self.reserve_temp_local();
        let span = self.reserve_temp_local();
        let separator = self.reserve_temp_local();
        let result = (|| {
            self.emit_intl_locale_unicode_bounds(
                tag,
                unicode_start,
                unicode_end,
                insertion,
                function,
            );
            self.emit_intl_locale_keyword_span(
                tag,
                option,
                unicode_start,
                unicode_end,
                start,
                end,
                function,
            );
            function.instruction(&Instruction::LocalGet(start));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(unicode_start));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(insertion));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(unicode_end));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalTee(start));
            function.instruction(&Instruction::LocalSet(end));
            function.instruction(&Instruction::End);
            self.emit_unpack_string_payload(tag, offset, length, function);
            self.emit_unpack_string_payload(value, scratch, size, function);
            function.instruction(&Instruction::LocalGet(size));
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(6));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(size));
            self.emit_heap_alloc_from_local(size, function)?;
            function.instruction(&Instruction::LocalSet(output));
            self.emit_intl_set_const(position, 0, function);
            self.emit_pack_string_payload(offset, start, function);
            function.instruction(&Instruction::LocalSet(span));
            self.emit_intl_locale_append_payload(span, output, position, function);
            function.instruction(&Instruction::LocalGet(unicode_start));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_set_const(span, self.strings.payload("-u"), function);
            self.emit_intl_locale_append_payload(span, output, position, function);
            function.instruction(&Instruction::End);
            self.emit_intl_write_separator(output, position, separator, function);
            self.emit_intl_set_const(span, self.strings.payload(option.key()), function);
            self.emit_intl_locale_append_payload(span, output, position, function);
            self.emit_intl_write_separator(output, position, separator, function);
            self.emit_intl_locale_append_payload(value, output, position, function);
            function.instruction(&Instruction::LocalGet(offset));
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(scratch));
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(size));
            self.emit_pack_string_payload(scratch, size, function);
            function.instruction(&Instruction::LocalSet(span));
            self.emit_intl_locale_append_payload(span, output, position, function);
            self.emit_pack_string_payload(output, position, function);
            function.instruction(&Instruction::LocalSet(tag));
            Ok(())
        })();
        for local in [
            separator,
            span,
            position,
            output,
            size,
            scratch,
            length,
            offset,
            end,
            start,
            insertion,
            unicode_end,
            unicode_start,
        ] {
            self.release_temp_local(local);
        }
        result
    }

    fn emit_intl_locale_extension_value(
        &mut self,
        tag: u32,
        option: LocaleExtensionOption,
        value: u32,
        function: &mut Function,
    ) {
        let unicode_start = self.reserve_temp_local();
        let unicode_end = self.reserve_temp_local();
        let insertion = self.reserve_temp_local();
        let start = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        self.emit_intl_locale_unicode_bounds(tag, unicode_start, unicode_end, insertion, function);
        self.emit_intl_locale_keyword_span(
            tag,
            option,
            unicode_start,
            unicode_end,
            start,
            end,
            function,
        );
        self.emit_intl_set_const(value, 0, function);
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(tag, offset, length, function);
        self.emit_intl_add_const(start, 3, function);
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_add_const(start, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(offset));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(length));
        self.emit_pack_string_payload(offset, length, function);
        function.instruction(&Instruction::LocalSet(value));
        function.instruction(&Instruction::End);
        for local in [
            length,
            offset,
            end,
            start,
            insertion,
            unicode_end,
            unicode_start,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_intl_locale_extension_getter(
        &mut self,
        option: LocaleExtensionOption,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let expected = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        self.emit_intl_locale_record_from_receiver(record, function)?;
        self.load_i64_to_local_from_offset(record, HEAP_INTL_LOCALE_TAG_OFFSET, tag, function);
        self.emit_intl_locale_extension_value(tag, option, value, function);
        if matches!(option, LocaleExtensionOption::Numeric) {
            self.emit_intl_set_const(self.result_local, 0, function);
            self.emit_intl_set_const(
                self.result_tag_local,
                ValueKind::Boolean.tag() as i64,
                function,
            );
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_unpack_string_payload(value, expected, length, function);
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64Eqz);
            self.emit_intl_set_const(expected, self.strings.payload("true"), function);
            self.emit_string_payload_equality_i32(value, expected, function);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::End);
        } else {
            self.emit_intl_locale_optional_string_result(value, function);
        }
        for local in [length, expected, value, tag, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_locale_optional_string_result(&mut self, value: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_intl_locale_variants_payload(
        &mut self,
        tag: u32,
        language: u32,
        script: u32,
        region: u32,
        base_name: u32,
        value: u32,
        function: &mut Function,
    ) {
        let prefix = self.reserve_temp_local();
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let base_length = self.reserve_temp_local();
        self.emit_intl_locale_prefix_length(
            language, script, region, prefix, offset, length, function,
        );
        self.emit_unpack_string_payload(base_name, offset, base_length, function);
        self.emit_intl_set_const(value, 0, function);
        function.instruction(&Instruction::LocalGet(base_length));
        function.instruction(&Instruction::LocalGet(prefix));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_add_const(prefix, 1, function);
        self.emit_unpack_string_payload(tag, offset, length, function);
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::LocalGet(prefix));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(offset));
        function.instruction(&Instruction::LocalGet(base_length));
        function.instruction(&Instruction::LocalGet(prefix));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(length));
        self.emit_pack_string_payload(offset, length, function);
        function.instruction(&Instruction::LocalSet(value));
        function.instruction(&Instruction::End);
        for local in [base_length, length, offset, prefix] {
            self.release_temp_local(local);
        }
    }

    pub(in crate::builtins) fn emit_intl_locale_variants_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let language = self.reserve_temp_local();
        let script = self.reserve_temp_local();
        let region = self.reserve_temp_local();
        let base_name = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        self.emit_intl_locale_record_from_receiver(record, function)?;
        for (offset, local) in [
            (HEAP_INTL_LOCALE_TAG_OFFSET, tag),
            (HEAP_INTL_LOCALE_LANGUAGE_OFFSET, language),
            (HEAP_INTL_LOCALE_SCRIPT_OFFSET, script),
            (HEAP_INTL_LOCALE_REGION_OFFSET, region),
            (HEAP_INTL_LOCALE_BASE_NAME_OFFSET, base_name),
        ] {
            self.load_i64_to_local_from_offset(record, offset, local, function);
        }
        self.emit_intl_locale_variants_payload(
            tag, language, script, region, base_name, value, function,
        );
        self.emit_intl_locale_optional_string_result(value, function);
        for local in [value, base_name, region, script, language, tag, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(in crate::builtins) fn emit_intl_locale_calendar_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Calendar, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_collation_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Collation, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_first_day_of_week_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::FirstDayOfWeek, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_hour_cycle_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::HourCycle, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_case_first_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::CaseFirst, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_numeric_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::Numeric, function)
    }

    pub(in crate::builtins) fn emit_intl_locale_numbering_system_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_locale_extension_getter(LocaleExtensionOption::NumberingSystem, function)
    }
}
