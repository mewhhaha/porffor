use super::*;
use crate::builtins::intl_provider_wire::IntlByteArrayReader;
use lila_intl::IntlOperation;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_locale_get_text_info(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let response = self.emit_intl_locale_provider_response(
            lila_intl::LocaleTextInfoOperation::HOST_OP,
            &tag,
            false,
            function,
        )?;
        let reader = IntlByteArrayReader::new(&response, schema, function);
        let word = schema.reserve_i64_local(function);
        reader.read_u64(word, schema, function);
        reader.finish(schema, function);
        word.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        self.emit_intl_locale_provider_fault_if(function);
        let value = schema.reserve_value_local(function);
        value.set_undefined(function);
        for (code, direction) in [(1, "ltr"), (2, "rtl")] {
            word.load(function);
            function.instruction(&Instruction::I64Const(code));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            let text = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(direction, function)?,
                function,
            );
            value.set_reference(&text, schema, function);
            text.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        let object = self.emit_intl_number_result_object(function)?;
        self.emit_intl_number_append_result_property(&object, "direction", &value, function)?;
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&object, schema, function);
        object.clear(function);
        value.clear(function);
        schema.release_i64_local(word, function);
        response.clear(function);
        tag.clear(function);
        record.clear(function);
        Ok(())
    }
}
