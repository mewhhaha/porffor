use super::*;
use lila_intl::{
    IntlOperation, LocaleTransformError, LocaleTransformRequest, LocaleTransformResult,
};

impl<'a> FunctionBuilder<'a> {
    /// Invoke the existing pure, pinned provider after AOT-owned validation and
    /// coercion. The operation marker restricts this ABI to locale transforms.
    pub(super) fn emit_intl_provider_locale_transform<O>(
        &mut self,
        tag_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError>
    where
        O: IntlOperation<
            Request = LocaleTransformRequest,
            Response = LocaleTransformResult,
            Error = LocaleTransformError,
        >,
    {
        let capacity = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let outcome = self.reserve_temp_local();
        // The first pure call determines the exact result size. No JavaScript
        // read or coercion can occur between this query and the writing call.
        function.instruction(&Instruction::I64Const(O::HOST_OP.wire()));
        function.instruction(&Instruction::LocalGet(tag_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(self.intl_call_import_function_index()?));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // A canonical locale is nonempty, so a zero-capacity query must report
        // a positive RequiredCapacity. Reject every other host response.
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(0).wire(),
        ));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(u32::MAX).wire(),
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(0).wire(),
        ));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(capacity));
        self.emit_heap_alloc_from_local(capacity, function)?;
        function.instruction(&Instruction::LocalSet(output));
        function.instruction(&Instruction::I64Const(O::HOST_OP.wire()));
        function.instruction(&Instruction::LocalGet(tag_payload_local));
        self.emit_pack_string_payload(output, capacity, function);
        function.instruction(&Instruction::Call(self.intl_call_import_function_index()?));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::LocalGet(capacity));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(output, capacity, function);
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        for local in [outcome, output, capacity] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Alias resolution can replace language, region, script, and variants.
    /// Reparse its result before exposing any of the cached component slots.
    pub(super) fn emit_intl_locale_canonicalize_components(
        &mut self,
        invocation: CanonicalLocaleTagInvocationLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (input, tag, language, script, region, base_name, valid) = invocation.into_parts();
        let canonical = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(input));
        function.instruction(&Instruction::LocalSet(canonical));
        self.emit_intl_provider_locale_transform::<lila_intl::CanonicalizeLocale>(
            canonical, function,
        )?;
        self.emit_intl_canonicalize_locale_tag(
            CanonicalLocaleTagInvocationLocals::new(
                CanonicalLocaleTagInputPayloadLocal::new(canonical),
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
        // The typed host result is required to be a structurally valid tag.
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(canonical);
        Ok(())
    }
}
