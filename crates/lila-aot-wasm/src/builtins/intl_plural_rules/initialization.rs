use super::super::intl_numberformat::{NfOptionsLocals, emit_plural_rules_digit_options};
use super::provider_wire::{
    PluralRulesOperation, PluralRulesResponseReader, PluralRulesWireField, PluralRulesWireWord,
};
use super::*;

#[must_use]
struct ReservedPluralRulesObject(u32);

#[must_use]
struct InitializedPluralRulesObject(u32);

impl FunctionBuilder<'_> {
    fn emit_reserve_plural_rules_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedPluralRulesObject, EmitError> {
        let object = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let result = (|| {
            self.compile_new_target_to_locals(prototype, tag, function)?;
            function.instruction(&Instruction::LocalGet(tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_pr_type_error(PR_MUST_USE_NEW, function)?;
            function.instruction(&Instruction::Else);
            self.emit_new_target_prototype_to_locals(
                INTL_PLURAL_RULES_PROTOTYPE_GLOBAL_INDEX,
                NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(
                    OrdinaryDefaultPrototype::IntlPluralRules,
                ),
                prototype,
                tag,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(prototype),
                Some(tag),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(object));
            Ok(())
        })();
        self.release_temp_local(tag);
        self.release_temp_local(prototype);
        if let Err(error) = result {
            self.release_temp_local(object);
            return Err(error);
        }
        Ok(ReservedPluralRulesObject(object))
    }

    fn emit_initialize_plural_rules_object(
        &self,
        reserved: ReservedPluralRulesObject,
        record: u32,
        function: &mut Function,
    ) -> InitializedPluralRulesObject {
        self.store_i64_const_at_offset(
            reserved.0,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_PLURAL_RULES,
            function,
        );
        self.store_i64_local_at_offset(
            reserved.0,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        InitializedPluralRulesObject(reserved.0)
    }

    fn emit_publish_plural_rules_object(
        &mut self,
        initialized: InitializedPluralRulesObject,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_pr_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        self.release_temp_local(initialized.0);
    }

    fn emit_plural_rules_locale_request(
        &mut self,
        locales: u32,
        matcher: u32,
        rule_type: u32,
        options: &NfOptionsLocals,
        request: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = [
            PluralRulesWireField::Word(PluralRulesWireWord::Local(matcher)),
            PluralRulesWireField::Word(PluralRulesWireWord::Constant(1)),
            PluralRulesWireField::CanonicalLocales(locales),
            PluralRulesWireField::RuleOptions { rule_type, options },
        ];
        self.emit_pr_provider_request(
            PluralRulesOperation::ResolveLocale,
            &fields,
            request,
            function,
        )
    }

    pub(crate) fn emit_intl_plural_rules_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reserved = self.emit_reserve_plural_rules_object(function)?;
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let matcher = self.reserve_temp_local();
        let rule_type = self.reserve_temp_local();
        let notation = self.reserve_temp_local();
        let compact_display = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let status = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let selected = NfOptionsLocals::reserve(self);

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_pr_canonical_locales(locales, requested.payload, requested.tag, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_pr_options_object(options, function)?;
        self.emit_pr_choice_option(
            options,
            "localeMatcher",
            &[("best fit", 2), ("lookup", 1)],
            2,
            matcher,
            function,
        )?;
        self.emit_pr_choice_option(
            options,
            "type",
            &[("cardinal", 1), ("ordinal", 2)],
            1,
            rule_type,
            function,
        )?;
        self.emit_pr_choice_option(
            options,
            "notation",
            &[
                ("standard", 1),
                ("scientific", 2),
                ("engineering", 3),
                ("compact", 4),
            ],
            1,
            notation,
            function,
        )?;
        // The proposal reads and validates compactDisplay for every notation.
        self.emit_pr_choice_option(
            options,
            "compactDisplay",
            &[("short", 1), ("long", 2)],
            1,
            compact_display,
            function,
        )?;

        for word in NfWord::ALL {
            self.emit_pr_set_const(selected.word(word), 0, function);
        }
        self.emit_pr_set_const(selected.word(NfWord::Style), 1, function);
        self.emit_pr_set_const(selected.word(NfWord::Notation), 1, function);
        self.emit_pr_set_const(selected.word(NfWord::RoundingIncrement), 1, function);
        self.emit_pr_set_const(selected.word(NfWord::RoundingMode), 7, function);
        self.emit_pr_set_const(selected.word(NfWord::Grouping), 1, function);
        self.emit_pr_set_const(selected.word(NfWord::SignDisplay), 1, function);
        function.instruction(&Instruction::LocalGet(notation));
        function.instruction(&Instruction::LocalSet(selected.word(NfWord::Notation)));
        self.emit_pr_if_eq(notation, 4, function);
        function.instruction(&Instruction::LocalGet(compact_display));
        function.instruction(&Instruction::LocalSet(
            selected.word(NfWord::CompactDisplay),
        ));
        function.instruction(&Instruction::End);
        emit_plural_rules_digit_options(self, options, &selected, function)?;

        self.emit_plural_rules_locale_request(
            requested.payload,
            matcher,
            rule_type,
            &selected,
            request,
            function,
        )?;
        self.emit_pr_provider_call(
            PluralRulesOperation::ResolveLocale,
            request,
            response,
            function,
        )?;
        let reader = PluralRulesResponseReader::new(
            self,
            response,
            PluralRulesOperation::ResolveLocale,
            function,
        )?;
        reader.word(self, status, function);
        self.emit_pr_if_eq(status, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(HEAP_INTL_PLURAL_RULES_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        self.emit_plural_rules_record_from_response(&reader, record, function);
        reader.finish(self, function);
        let initialized = self.emit_initialize_plural_rules_object(reserved, record, function);

        selected.release(self);
        for local in [
            record,
            status,
            response,
            request,
            compact_display,
            notation,
            rule_type,
            matcher,
            value.tag,
            value.payload,
            requested.tag,
            requested.payload,
            options.tag,
            options.payload,
            locales.tag,
            locales.payload,
        ] {
            self.release_temp_local(local);
        }
        self.emit_publish_plural_rules_object(initialized, function);
        Ok(())
    }

    pub(crate) fn emit_intl_plural_rules_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let matcher = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let status = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_pr_canonical_locales(locales, requested.payload, requested.tag, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_pr_options_object(options, function)?;
        self.emit_pr_choice_option(
            options,
            "localeMatcher",
            &[("best fit", 2), ("lookup", 1)],
            2,
            matcher,
            function,
        )?;
        self.emit_pr_provider_request(
            PluralRulesOperation::ResolveLocale,
            &[
                PluralRulesWireField::Word(PluralRulesWireWord::Local(matcher)),
                PluralRulesWireField::Word(PluralRulesWireWord::Constant(2)),
                PluralRulesWireField::CanonicalLocales(requested.payload),
            ],
            request,
            function,
        )?;
        self.emit_pr_provider_call(
            PluralRulesOperation::ResolveLocale,
            request,
            response,
            function,
        )?;
        let reader = PluralRulesResponseReader::new(
            self,
            response,
            PluralRulesOperation::ResolveLocale,
            function,
        )?;
        reader.word(self, status, function);
        self.emit_pr_if_eq(status, 2, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        reader.word(self, count, function);
        reader.require_records(self, count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_pr_set_const(index, 0, function);
        self.emit_pr_set_const(value.tag, ValueKind::String.tag() as i64, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.bytes(self, value.payload, function);
        self.emit_array_write(output, index, value.payload, value.tag, function)?;
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_pr_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );

        for local in [
            value.tag,
            value.payload,
            output,
            index,
            count,
            status,
            response,
            request,
            matcher,
            requested.tag,
            requested.payload,
            options.tag,
            options.payload,
            locales.tag,
            locales.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
