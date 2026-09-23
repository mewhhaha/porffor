use super::*;

impl FunctionBuilder<'_> {
    /// Both callers have already extracted thisNumberValue/thisBigIntValue.
    /// Direct intrinsic calls preserve that value across locales/options hooks
    /// and do not consult a mutable Intl property or format accessor.
    pub(crate) fn emit_intrinsic_number_locale_format(
        &mut self,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let formatter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let format = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_nf_intrinsic_call(
            StandardBuiltinId::IntlNumberFormatConstructor,
            None,
            &[
                (locales.payload, locales.tag),
                (options.payload, options.tag),
            ],
            formatter.payload,
            formatter.tag,
            function,
        )?;
        self.emit_nf_intrinsic_call(
            StandardBuiltinId::IntlNumberFormatPrototypeFormatGetter,
            Some((formatter.payload, Some(formatter.tag))),
            &[],
            format.payload,
            format.tag,
            function,
        )?;
        self.emit_function_handle_call(
            format.payload,
            format.tag,
            None,
            &[(value.payload, value.tag)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        for local in [
            format.tag,
            format.payload,
            formatter.tag,
            formatter.payload,
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
