use super::extension_options::LocaleExtensionOption;
use super::*;
use crate::builtins::intl_provider_wire::IntlByteArrayReader;
use lila_intl::IntlOperation;
impl FunctionBuilder<'_> {
    fn emit_intl_locale_numbering_system_array(
        &mut self,
        tag: &GcLocal<StringValue>,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let response = self.emit_intl_locale_provider_response(
            lila_intl::LocaleNumberingSystemsOperation::HOST_OP,
            tag,
            false,
            function,
        )?;
        let reader = IntlByteArrayReader::new(&response, schema, function);
        let text = reader.consume_remaining_utf8(schema, function);
        reader.finish(schema, function);
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&text, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        self.emit_intl_locale_range(length, 3, 8, function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_intl_locale_provider_fault_if(function);
        set_i32(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        self.emit_intl_locale_range(unit, b'a' as i32, b'z' as i32, function);
        self.emit_intl_locale_range(unit, b'0' as i32, b'9' as i32, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.emit_intl_locale_provider_fault_if(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_intl_locale_info_string_singleton(&text, output, function)?;
        for local in [unit, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        text.clear(function);
        response.clear(function);
        Ok(())
    }
    pub(crate) fn emit_intl_locale_get_numbering_systems(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_intl_locale_record_from_receiver(function)?;
        let tag = self.emit_intl_locale_tag(&record, function);
        let explicit = self.emit_intl_locale_extension_value(
            &tag,
            LocaleExtensionOption::NumberingSystem,
            function,
        );
        let output = schema.reserve_value_local(function);
        explicit.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_locale_numbering_system_array(&tag, &output, function)?;
        function.instruction(&Instruction::Else);
        let text = schema.reserve_gc_local(function).initialize(
            explicit.load(schema, function).require_non_null(function),
            function,
        );
        self.emit_intl_locale_info_string_singleton(&text, &output, function)?;
        text.clear(function);
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
