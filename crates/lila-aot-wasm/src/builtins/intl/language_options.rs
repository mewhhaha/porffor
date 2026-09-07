//! The language/script/region part of Intl.Locale's ordered options pass.
//!
//! Observable operations remain in the shared object/coercion emitters. This
//! module only emits subtag validation and reconstruction of an already-valid
//! tag; it does not evaluate JavaScript or introduce a locale data provider.

use super::*;

/// CoerceOptionsToObject has completed. Undefined represents the fresh, empty,
/// null-prototype options object: no operation can observe that object's
/// identity, and none of its properties can exist. Every other value here is
/// the actual boxed/object receiver, not a copy of its properties.
#[must_use]
pub(super) struct CoercedIntlLocaleOptions(TaggedLocals);

#[derive(Clone, Copy)]
enum LanguageOption {
    Language,
    Script,
    Region,
}

impl LanguageOption {
    const fn property(self) -> &'static str {
        match self {
            Self::Language => "language",
            Self::Script => "script",
            Self::Region => "region",
        }
    }
}

/// Only the subtag validator can construct the value used to replace a field.
#[must_use]
struct ValidatedLocaleComponent(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_intl_locale_coerce_options(
        &mut self,
        destination: TaggedLocals,
        function: &mut Function,
    ) -> Result<CoercedIntlLocaleOptions, EmitError> {
        let argument_payload = self.reserve_temp_local();
        let argument_tag = self.reserve_temp_local();
        let result = (|| {
            self.emit_builtin_arg_to_locals(1, argument_payload, argument_tag, function);
            self.emit_intl_set_const(destination.payload, 0, function);
            self.emit_intl_set_const(destination.tag, ValueKind::Undefined.tag() as i64, function);
            function.instruction(&Instruction::LocalGet(argument_tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.compile_nullish_tagged_i32(argument_tag, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Intl.Locale options must not be null",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            self.emit_value_to_current_function_realm_object_locals(
                argument_payload,
                argument_tag,
                destination.payload,
                destination.tag,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
            Ok(CoercedIntlLocaleOptions(destination))
        })();
        self.release_temp_local(argument_tag);
        self.release_temp_local(argument_payload);
        result
    }

    /// Apply core overrides without losing variants, other extensions, or
    /// private use. The original suffix boundary is captured before any field
    /// is replaced. Reusing the structural canonicalizer keeps all five
    /// represented Locale slots consistent, including script/region casing.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_intl_locale_language_options(
        &mut self,
        options: CoercedIntlLocaleOptions,
        tag: u32,
        language: u32,
        script: u32,
        region: u32,
        base_name: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options = options.0;
        let source_offset = self.reserve_temp_local();
        let source_length = self.reserve_temp_local();
        let prefix_length = self.reserve_temp_local();
        let scratch_offset = self.reserve_temp_local();
        let scratch_length = self.reserve_temp_local();
        let suffix = self.reserve_temp_local();
        let suffix_length = self.reserve_temp_local();
        let changed = self.reserve_temp_local();
        let output_size = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let position = self.reserve_temp_local();
        let separator = self.reserve_temp_local();
        let rebuilt_tag = self.reserve_temp_local();
        let valid = self.reserve_temp_local();
        let result = (|| {
            function.instruction(&Instruction::LocalGet(options.tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));

            self.emit_unpack_string_payload(tag, source_offset, source_length, function);
            self.emit_intl_locale_prefix_length(
                language,
                script,
                region,
                prefix_length,
                scratch_offset,
                scratch_length,
                function,
            );
            function.instruction(&Instruction::LocalGet(source_length));
            function.instruction(&Instruction::LocalGet(prefix_length));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(suffix_length));
            function.instruction(&Instruction::LocalGet(source_offset));
            function.instruction(&Instruction::LocalGet(prefix_length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalGet(suffix_length));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(suffix));

            self.emit_intl_set_const(changed, 0, function);
            // Each Get, ToString and validation completes before the next Get.
            for (option, destination) in [
                (LanguageOption::Language, language),
                (LanguageOption::Script, script),
                (LanguageOption::Region, region),
            ] {
                self.emit_intl_locale_language_option(
                    options,
                    option,
                    destination,
                    changed,
                    function,
                )?;
            }

            function.instruction(&Instruction::LocalGet(changed));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_locale_prefix_length(
                language,
                script,
                region,
                prefix_length,
                scratch_offset,
                scratch_length,
                function,
            );
            function.instruction(&Instruction::LocalGet(prefix_length));
            function.instruction(&Instruction::LocalGet(suffix_length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(output_size));
            self.emit_heap_alloc_from_local(output_size, function)?;
            function.instruction(&Instruction::LocalSet(output));
            self.emit_intl_set_const(position, 0, function);
            self.emit_intl_locale_append_payload(language, output, position, function);
            for component in [script, region] {
                function.instruction(&Instruction::LocalGet(component));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_intl_write_separator(output, position, separator, function);
                self.emit_intl_locale_append_payload(component, output, position, function);
                function.instruction(&Instruction::End);
            }
            // The retained suffix already includes its leading separator.
            self.emit_intl_locale_append_payload(suffix, output, position, function);
            function.instruction(&Instruction::LocalGet(output));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalGet(position));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(rebuilt_tag));
            self.emit_intl_canonicalize_locale_tag(
                CanonicalLocaleTagInvocationLocals::new(
                    CanonicalLocaleTagInputPayloadLocal::new(rebuilt_tag),
                    CanonicalLocaleTagPayloadLocal::new(tag),
                    CanonicalLocaleLanguagePayloadLocal::new(language),
                    CanonicalLocaleScriptPayloadLocal::new(script),
                    CanonicalLocaleRegionPayloadLocal::new(region),
                    CanonicalLocaleBaseNamePayloadLocal::new(base_name),
                    CanonicalLocaleValidityLocal::new(valid),
                ),
                function,
            )?;
            function.instruction(&Instruction::LocalGet(valid));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid language tag after Intl.Locale options",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            Ok(())
        })();
        for local in [
            valid,
            rebuilt_tag,
            separator,
            position,
            output,
            output_size,
            changed,
            suffix_length,
            suffix,
            scratch_length,
            scratch_offset,
            prefix_length,
            source_length,
            source_offset,
        ] {
            self.release_temp_local(local);
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_intl_locale_prefix_length(
        &mut self,
        language: u32,
        script: u32,
        region: u32,
        prefix_length: u32,
        scratch_offset: u32,
        scratch_length: u32,
        function: &mut Function,
    ) {
        self.emit_unpack_string_payload(language, scratch_offset, prefix_length, function);
        for component in [script, region] {
            function.instruction(&Instruction::LocalGet(component));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_unpack_string_payload(component, scratch_offset, scratch_length, function);
            function.instruction(&Instruction::LocalGet(prefix_length));
            function.instruction(&Instruction::LocalGet(scratch_length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(prefix_length));
            function.instruction(&Instruction::End);
        }
    }

    fn emit_intl_locale_language_option(
        &mut self,
        options: TaggedLocals,
        option: LanguageOption,
        destination: u32,
        changed: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let value_tag = self.reserve_temp_local();
        let result = (|| {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(option.property()),
            ));
            function.instruction(&Instruction::LocalSet(key));
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
            self.emit_value_to_string_payload(value, value_tag, function)?;
            function.instruction(&Instruction::LocalSet(value));
            self.emit_return_current_completion_if_throw(function);
            let validated = self.emit_intl_locale_validate_component(option, value, function)?;
            function.instruction(&Instruction::LocalGet(validated.0));
            function.instruction(&Instruction::LocalSet(destination));
            self.emit_intl_set_const(changed, 1, function);
            function.instruction(&Instruction::End);
            Ok(())
        })();
        self.release_temp_local(value_tag);
        self.release_temp_local(value);
        self.release_temp_local(key);
        result
    }

    fn emit_intl_locale_validate_component(
        &mut self,
        option: LanguageOption,
        value: u32,
        function: &mut Function,
    ) -> Result<ValidatedLocaleComponent, EmitError> {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let folded = self.reserve_temp_local();
        let all_alpha = self.reserve_temp_local();
        let all_digit = self.reserve_temp_local();
        let result = (|| {
            self.emit_unpack_string_payload(value, offset, length, function);
            // Reject length before scanning: at most eight bytes are read.
            match option {
                LanguageOption::Language => {
                    self.emit_intl_locale_in_range(length, 2, 3, function);
                    self.emit_intl_locale_in_range(length, 5, 8, function);
                    function.instruction(&Instruction::I32Or);
                }
                LanguageOption::Script => {
                    self.emit_intl_locale_in_range(length, 4, 4, function);
                }
                LanguageOption::Region => {
                    self.emit_intl_locale_in_range(length, 2, 3, function);
                }
            }
            self.emit_intl_locale_component_guard(option, function)?;
            self.emit_intl_set_const(index, 0, function);
            self.emit_intl_set_const(all_alpha, 1, function);
            self.emit_intl_set_const(all_digit, 1, function);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            self.emit_load_string_byte(offset, index, byte, function);
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(folded));
            function.instruction(&Instruction::LocalGet(all_alpha));
            self.emit_intl_locale_in_range(folded, b'a' as i64, b'z' as i64, function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(all_alpha));
            function.instruction(&Instruction::LocalGet(all_digit));
            self.emit_intl_locale_in_range(byte, b'0' as i64, b'9' as i64, function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(all_digit));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(all_alpha));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            match option {
                LanguageOption::Language | LanguageOption::Script => {}
                LanguageOption::Region => {
                    self.emit_intl_locale_in_range(length, 2, 2, function);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::LocalGet(all_digit));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    self.emit_intl_locale_in_range(length, 3, 3, function);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::I32Or);
                }
            }
            self.emit_intl_locale_component_guard(option, function)?;
            Ok(ValidatedLocaleComponent(value))
        })();
        for local in [all_digit, all_alpha, folded, byte, index, length, offset] {
            self.release_temp_local(local);
        }
        result
    }

    /// Consume an i32 validity condition; no invalid subtag can fall through.
    fn emit_intl_locale_component_guard(
        &mut self,
        option: LanguageOption,
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

    fn emit_intl_locale_in_range(
        &self,
        value: u32,
        minimum: i64,
        maximum: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(minimum));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(maximum));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_intl_locale_append_payload(
        &mut self,
        payload: u32,
        output: u32,
        position: u32,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        self.emit_unpack_string_payload(payload, offset, length, function);
        self.emit_intl_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset, index, byte, function);
        self.emit_intl_store_byte(output, position, byte, function);
        for local in [index, position] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(byte);
        self.release_temp_local(index);
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }
}
