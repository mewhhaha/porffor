use super::*;
use crate::builtins::intl_datetimeformat::IntlDateTimeFormatPurpose;

pub(crate) enum DateLocaleFormat {
    Date,
    Time,
    DateAndTime,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_date_to_locale_string(
        &mut self,
        format: DateLocaleFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload = self.reserve_temp_local();
        let receiver_tag = self.reserve_temp_local();
        let time_payload = self.reserve_temp_local();
        let time_tag = self.reserve_temp_local();
        let formatter_payload = self.reserve_temp_local();
        let formatter_tag = self.reserve_temp_local();
        let format_payload = self.reserve_temp_local();
        let format_tag = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload, receiver_tag, function)?;
        self.emit_date_value_payload(receiver_payload, receiver_tag, time_payload, function)?;
        function.instruction(&Instruction::LocalGet(time_payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Invalid Date")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);

        // Keep DateValue before any locale/options callbacks can mutate the Date.
        self.emit_intl_create_date_time_format(
            IntlDateTimeFormatPurpose::DateLocale(format),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(formatter_payload));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(formatter_tag));

        // The intrinsic getter and bound formatter reuse the shared field walk;
        // replacements of the global Intl object or public accessor are ignored.
        let format_getter = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Intl.DateTimeFormat format getter"))?;
        self.emit_direct_js_call(
            &format_getter,
            Some((formatter_payload, Some(formatter_tag))),
            &[],
            format_payload,
            format_tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(time_tag));
        self.emit_function_handle_call(
            format_payload,
            format_tag,
            None,
            &[(time_payload, time_tag)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            format_tag,
            format_payload,
            formatter_tag,
            formatter_payload,
            time_tag,
            time_payload,
            receiver_tag,
            receiver_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
