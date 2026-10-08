use super::*;
use lila_intl::{
    IntlOperation, LocaleTransformError, LocaleTransformRequest, LocaleTransformResult,
};
impl FunctionBuilder<'_> {
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
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let transformed = self.emit_intl_provider_locale_transform::<O>(
            &tag,
            RuntimeErrorMessage::INVALID_LANGUAGE_TAG,
            function,
        )?;
        tag.clear(function);
        record.clear(function);
        let components = self.emit_intl_locale_components(transformed, function)?;
        let reserved = self.emit_reserve_intrinsic_intl_locale_object(function)?;
        let initialized = self.emit_initialize_intl_locale_object(reserved, &components, function);
        components.clear(function);
        self.emit_publish_intl_locale_object(initialized, function);
        Ok(())
    }
}
