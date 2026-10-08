use super::extension_options::LocaleExtensionOption;
use super::locale_information_list::LocaleInformationKind;
use super::*;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_locale_get_calendars(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let explicit =
            self.emit_intl_locale_extension_value(&tag, LocaleExtensionOption::Calendar, function);
        let output = schema.reserve_value_local(function);
        explicit.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_locale_information_array(
            LocaleInformationKind::Calendars,
            &tag,
            &output,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let value = schema.reserve_gc_local(function).initialize(
            explicit.load(schema, function).require_non_null(function),
            function,
        );
        self.emit_intl_locale_info_string_singleton(&value, &output, function)?;
        value.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().initialize(function);
        self.completion().value().copy_from(&output, function);
        output.clear(function);
        explicit.clear(function);
        tag.clear(function);
        record.clear(function);
        Ok(())
    }
}
