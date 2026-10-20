use super::*;
use crate::builtins::intl_datetimeformat::IntlDateTimeFormatPurpose;

pub(crate) enum DateLocaleFormat {
    Date,
    Time,
    DateAndTime,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_date_to_locale_string(
        &mut self,
        format: DateLocaleFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let time = schema.reserve_value_local(function);
        let formatter = schema.reserve_completion(function);
        let callable = schema.reserve_completion(function);
        let result = schema.reserve_completion(function);
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                // Copy before locale/options callbacks can mutate the receiver.
                time.set_number(finite.payload(), function);
                let cleanup = builder.open_frame(ControlFrameKind::Block, function);
                builder.emit_intl_create_date_time_format(
                    IntlDateTimeFormatPurpose::DateLocale(format),
                    function,
                )?;
                formatter.copy_from(builder.completion(), function);
                builder.emit_date_native_throw_exit(&formatter, &result, cleanup, function);
                let getter = builder
                    .functions
                    .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported("missing Intl.DateTimeFormat format getter")
                    })?;
                builder.emit_direct_js_call(
                    &getter,
                    Some(formatter.value()),
                    &[],
                    &callable,
                    function,
                )?;
                builder.emit_date_native_throw_exit(&callable, &result, cleanup, function);
                receiver.set_undefined(function);
                let arguments = builder.emit_pre_evaluated_arg_vector(&[&time], function);
                builder.emit_function_or_proxy_call_with_argv(
                    callable.value(),
                    &receiver,
                    &arguments,
                    &result,
                    function,
                )?;
                arguments.clear(function);
                builder.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                Ok(())
            },
            |builder, function| {
                let string = schema.reserve_gc_local(function).initialize(
                    builder.emit_interned_string_reference("Invalid Date", function)?,
                    function,
                );
                time.set_reference(&string, schema, function);
                result.set_normal(&time, function);
                string.clear(function);
                Ok(())
            },
        )?;
        self.completion().copy_from(&result, function);
        captured.release(self, function);
        result.clear(function);
        callable.clear(function);
        formatter.clear(function);
        time.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
