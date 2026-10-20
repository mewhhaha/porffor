use super::locale_information_list::LocaleInformationKind;
use super::*;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_locale_get_time_zones(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let region = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IntlLocaleObject>()
                .field(IntlLocaleObjectSchema::REGION)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        let output = schema.reserve_value_local(function);
        output.set_undefined(function);
        region.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let tag = self.emit_intl_locale_tag(&record, function);
        self.emit_intl_locale_information_array(
            LocaleInformationKind::TimeZones,
            &tag,
            &output,
            function,
        )?;
        tag.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().initialize(function);
        self.completion().value().copy_from(&output, function);
        output.clear(function);
        region.clear(function);
        record.clear(function);
        Ok(())
    }
}
