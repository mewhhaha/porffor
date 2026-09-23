use super::*;
use lila_intl::{
    IntlOperation, LocaleTransformError, LocaleTransformRequest, LocaleTransformResult,
};

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_intl_locale_likely_subtags_builtin<O>(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError>
    where
        O: IntlOperation<
            Request = LocaleTransformRequest,
            Response = LocaleTransformResult,
            Error = LocaleTransformError,
        >,
    {
        let record = self.reserve_temp_local();
        let transformed = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let language = self.reserve_temp_local();
        let script = self.reserve_temp_local();
        let region = self.reserve_temp_local();
        let base_name = self.reserve_temp_local();
        let valid = self.reserve_temp_local();

        self.emit_intl_locale_record_from_receiver(record, function)?;
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_LOCALE_TAG_OFFSET,
            transformed,
            function,
        );
        self.emit_intl_provider_locale_transform::<O>(transformed, function)?;
        self.emit_intl_canonicalize_locale_tag(
            CanonicalLocaleTagInvocationLocals::new(
                CanonicalLocaleTagInputPayloadLocal::new(transformed),
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
        // Host results have already crossed the canonical identifier boundary.
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        let reserved = self.emit_reserve_intrinsic_intl_locale_object(function)?;
        let initialized = self.emit_initialize_intl_locale_object(
            reserved, tag, language, script, region, base_name, function,
        )?;
        self.emit_publish_intl_locale_object(initialized, function);
        for local in [
            valid,
            base_name,
            region,
            script,
            language,
            tag,
            transformed,
            record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
